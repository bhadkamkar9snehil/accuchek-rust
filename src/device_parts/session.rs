fn operate_device<T: UsbContext>(
    device: &rusb::Device<T>,
    accu_chek: &AccuChekDevice,
) -> Result<DownloadResult, AccuChekError> {
    let handle = device.open()?;

    #[cfg(unix)]
    {
        if handle.kernel_driver_active(accu_chek.interface_number).unwrap_or(false) {
            handle.detach_kernel_driver(accu_chek.interface_number)?;
        }
    }

    // Avoid re-selecting an already-active configuration. This mirrors the maintained
    // WebUSB driver and is friendlier to WinUSB, where redundant SET_CONFIGURATION calls can
    // fail on otherwise healthy devices.
    if handle.active_configuration()? != accu_chek.config_value {
        handle.set_active_configuration(accu_chek.config_value)?;
    }
    handle.claim_interface(accu_chek.interface_number)?;
    handle.set_alternate_setting(accu_chek.interface_number, accu_chek.alternate_setting)?;

    let mut associated = false;

    let bulk_out = |name: &str, data: &[u8]| -> Result<(), AccuChekError> {
        info!("sending {} ({} bytes)", name, data.len());
        let written = handle
            .write_bulk(accu_chek.send_endpoint, data, USB_TIMEOUT)
            .map_err(|e| AccuChekError::Communication(format!("{} write failed: {}", name, e)))?;
        if written != data.len() {
            return Err(AccuChekError::Communication(format!(
                "{} short write: {} of {} bytes",
                name,
                written,
                data.len()
            )));
        }
        Ok(())
    };

    let bulk_in = |name: &str, max_len: usize| -> Result<Vec<u8>, AccuChekError> {
        let mut buffer = vec![0u8; max_len];
        info!("receiving {}", name);
        let read = handle
            .read_bulk(accu_chek.receive_endpoint, &mut buffer, USB_TIMEOUT)
            .map_err(|e| AccuChekError::Communication(format!("{} read failed: {}", name, e)))?;
        buffer.truncate(read);
        if buffer.is_empty() {
            return Err(AccuChekError::Communication(format!("{} returned no data", name)));
        }
        Ok(buffer)
    };

    let operation = (|| -> Result<DownloadResult, AccuChekError> {
        let mut control = [0u8; 2];
        handle
            .read_control(
                rusb::request_type(
                    rusb::Direction::In,
                    rusb::RequestType::Standard,
                    rusb::Recipient::Device,
                ),
                rusb::constants::LIBUSB_REQUEST_GET_STATUS,
                0,
                0,
                &mut control,
                USB_TIMEOUT,
            )
            .map_err(|e| AccuChekError::Communication(format!("initial control transfer failed: {}", e)))?;

        let association_request = bulk_in("association request", 64)?;
        require_len(&association_request, 4, "association request")?;
        if u16_at(&association_request, 0, "association request type")?
            != APDU_TYPE_ASSOCIATION_REQUEST
        {
            return Err(AccuChekError::UnexpectedResponse);
        }

        bulk_out("association response", &build_association_response())?;
        associated = true;

        // Tidepool retries this stage once because some meters occasionally return a config
        // frame that cannot be parsed immediately after association.
        let mut config_frame = bulk_in("configuration report", 1024)?;
        if find_object(&config_frame, MDC_MOC_VMO_PMSTORE).is_err() {
            warn!("configuration report was not parseable; retrying association/config once");
            bulk_out("association response retry", &build_association_response())?;
            config_frame = bulk_in("configuration report retry", 1024)?;
        }
        // Parse again after the optional retry so no borrowed slice survives a frame replacement.
        let (pm_store_payload, attribute_count, pm_store_handle) =
            find_object(&config_frame, MDC_MOC_VMO_PMSTORE)?;
        let segment_count = find_attribute(pm_store_payload, attribute_count, MDC_ATTR_NUM_SEG)
            .and_then(|value| u16_at(value, 0, "segment count"))?;
        info!("meter reports {} stored segment(s)", segment_count);
        let mut invoke_id = u16_at(&config_frame, 6, "configuration invoke id")?;

        bulk_out("configuration confirmation", &build_config_response(invoke_id))?;
        bulk_out("MDS attribute request", &build_mds_request(invoke_id))?;

        let mds_frame = bulk_in("MDS attribute response", 1024)?;
        require_len(&mds_frame, 10, "MDS attribute response")?;
        if u16_at(&mds_frame, 0, "MDS APDU type")? == APDU_TYPE_ASSOCIATION_ABORT {
            return Err(AccuChekError::AssociationAborted);
        }
        invoke_id = u16_at(&mds_frame, 6, "MDS invoke id")?;
        let metadata = metadata_from_mds(accu_chek, &mds_frame);

        bulk_out(
            "segment information request",
            &build_segment_info_request(invoke_id, pm_store_handle),
        )?;
        let segment_info = bulk_in("segment information response", 1024)?;
        require_len(&segment_info, 8, "segment information response")?;
        invoke_id = u16_at(&segment_info, 6, "segment information invoke id")?;

        bulk_out(
            "data transfer request",
            &build_data_transfer_request(invoke_id, pm_store_handle),
        )?;
        let transfer_header = bulk_in("data transfer response", 1024)?;
        require_len(&transfer_header, 16, "data transfer response")?;
        if transfer_header.len() == 22 {
            let response = u16_at(&transfer_header, 20, "data transfer result")?;
            if response == 3 {
                info!("meter contains no transferable records");
                return Ok(DownloadResult {
                    device: metadata,
                    readings: Vec::new(),
                });
            }
            if response != 0 {
                return Err(protocol_error(
                    "data transfer response",
                    format!("meter returned result code {}", response),
                ));
            }
        }
        if transfer_header.len() < 22
            || u16_at(&transfer_header, 14, "data transfer action")?
                != ACTION_TYPE_MDC_ACT_SEG_TRIG_XFER
        {
            return Err(AccuChekError::UnexpectedResponse);
        }

        let mut readings = Vec::new();
        let mut next_id = 0usize;
        for page_index in 0..MAX_DATA_PAGES {
            let page = bulk_in("data page", 1024)?;
            require_len(&page, 33, "data page")?;
            let data_invoke_id = u16_at(&page, 6, "data page invoke id")?;
            let u0 = u32_at(&page, 22, "segment result field 0")?;
            let u1 = u32_at(&page, 26, "segment result field 1")?;
            let entries = u16_at(&page, 30, "segment entry count")?;
            let status = page[32];

            let parsed = parse_page(&page, &metadata.device_key, &mut next_id)?;
            readings.extend(parsed);

            bulk_out(
                "data page confirmation",
                &build_data_confirmation(data_invoke_id, pm_store_handle, u0, u1, entries),
            )?;

            if status & 0x40 != 0 {
                info!("completed data transfer after {} page(s)", page_index + 1);
                return Ok(DownloadResult { device: metadata, readings });
            }
        }

        Err(protocol_error(
            "data transfer",
            format!("exceeded {} pages without end-of-transfer flag", MAX_DATA_PAGES),
        ))
    })();

    if associated {
        let release = build_release_request();
        if let Err(error) = handle.write_bulk(accu_chek.send_endpoint, &release, USB_TIMEOUT) {
            warn!("best-effort association release write failed: {}", error);
        } else {
            let mut response = [0u8; 64];
            if let Err(error) =
                handle.read_bulk(accu_chek.receive_endpoint, &mut response, USB_TIMEOUT)
            {
                warn!("best-effort association release response failed: {}", error);
            }
        }
    }

    if let Err(error) = handle.release_interface(accu_chek.interface_number) {
        warn!("failed to release USB interface cleanly: {}", error);
    }

    operation
}

/// Rich sync API for the new desktop frontend.
pub fn find_and_download_accuchek(
    context: &Context,
    config: &Config,
    device_index: Option<usize>,
) -> Result<DownloadResult, AccuChekError> {
    let devices = context.devices()?;
    let mut valid_devices = Vec::new();

    for device in devices.iter() {
        if let Some(accu_chek) = check_device(&device, config) {
            valid_devices.push((device, accu_chek));
        }
    }

    if valid_devices.is_empty() {
        return Err(AccuChekError::NoDeviceFound);
    }

    let selected_index = device_index.unwrap_or(0);
    if selected_index >= valid_devices.len() {
        return Err(AccuChekError::InvalidDeviceIndex(selected_index));
    }

    let (device, accu_chek) = &valid_devices[selected_index];
    accu_chek.show(&format!("selected Accu-Chek device #{}", selected_index));
    operate_device(device, accu_chek)
}

/// Compatibility API used by the existing CLI/egui application.
pub fn find_and_operate_accuchek(
    context: &Context,
    config: &Config,
    device_index: Option<usize>,
) -> Result<Vec<GlucoseReading>, AccuChekError> {
    Ok(find_and_download_accuchek(context, config, device_index)?.readings)
}
