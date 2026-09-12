fn build_association_response() -> Vec<u8> {
    let mut msg = Vec::with_capacity(48);
    write_be16(&mut msg, APDU_TYPE_ASSOCIATION_RESPONSE);
    write_be16(&mut msg, 44);
    write_be16(&mut msg, 0x0003); // accepted-unknown-config
    write_be16(&mut msg, 20601); // data-proto-id
    write_be16(&mut msg, 38); // data-proto-info length
    write_be32(&mut msg, 0x80000002); // protocolVersion
    write_be16(&mut msg, 0x8000); // MDER
    write_be32(&mut msg, 0x80000000); // nomenclatureVersion
    write_be32(&mut msg, 0); // functionalUnits
    write_be32(&mut msg, 0x80000000); // manager
    write_be16(&mut msg, 8); // system-id length
    write_be32(&mut msg, 0x12345678);
    write_be32(&mut msg, 0x87654321);
    msg.resize(48, 0);
    msg
}

fn build_config_response(invoke_id: u16) -> Vec<u8> {
    let mut msg = Vec::with_capacity(26);
    write_be16(&mut msg, APDU_TYPE_PRESENTATION_APDU);
    write_be16(&mut msg, 22);
    write_be16(&mut msg, 20);
    write_be16(&mut msg, invoke_id);
    write_be16(&mut msg, DATA_APDU_RESPONSE_CONFIRMED_EVENT_REPORT);
    write_be16(&mut msg, 14);
    write_be16(&mut msg, 0);
    write_be32(&mut msg, 0);
    write_be16(&mut msg, EVENT_TYPE_MDC_NOTI_CONFIG);
    write_be16(&mut msg, 4);
    write_be16(&mut msg, 0x4000);
    write_be16(&mut msg, 0);
    msg
}

fn build_mds_request(invoke_id: u16) -> Vec<u8> {
    let mut msg = Vec::with_capacity(18);
    write_be16(&mut msg, APDU_TYPE_PRESENTATION_APDU);
    write_be16(&mut msg, 14);
    write_be16(&mut msg, 12);
    write_be16(&mut msg, invoke_id.wrapping_add(1));
    write_be16(&mut msg, DATA_APDU_INVOKE_GET);
    write_be16(&mut msg, 6);
    write_be16(&mut msg, 0); // MDS handle
    write_be16(&mut msg, 0); // attribute-id-list.count
    write_be16(&mut msg, 0); // attribute-id-list.length
    msg
}

fn build_segment_info_request(invoke_id: u16, pm_store_handle: u16) -> Vec<u8> {
    let mut msg = Vec::with_capacity(24);
    write_be16(&mut msg, APDU_TYPE_PRESENTATION_APDU);
    write_be16(&mut msg, 20);
    write_be16(&mut msg, 18);
    write_be16(&mut msg, invoke_id.wrapping_add(1));
    write_be16(&mut msg, DATA_APDU_INVOKE_CONFIRMED_ACTION);
    write_be16(&mut msg, 12);
    write_be16(&mut msg, pm_store_handle);
    write_be16(&mut msg, ACTION_TYPE_MDC_ACT_SEG_GET_INFO);
    write_be16(&mut msg, 6);
    write_be16(&mut msg, 1); // all segments
    write_be16(&mut msg, 2);
    write_be16(&mut msg, 0);
    msg
}

fn build_data_transfer_request(invoke_id: u16, pm_store_handle: u16) -> Vec<u8> {
    let mut msg = Vec::with_capacity(20);
    write_be16(&mut msg, APDU_TYPE_PRESENTATION_APDU);
    write_be16(&mut msg, 16);
    write_be16(&mut msg, 14);
    write_be16(&mut msg, invoke_id.wrapping_add(1));
    write_be16(&mut msg, DATA_APDU_INVOKE_CONFIRMED_ACTION);
    write_be16(&mut msg, 8);
    write_be16(&mut msg, pm_store_handle);
    write_be16(&mut msg, ACTION_TYPE_MDC_ACT_SEG_TRIG_XFER);
    write_be16(&mut msg, 2);
    write_be16(&mut msg, 0); // segment 0 = meter's active/all-data transfer
    msg
}

fn build_data_confirmation(
    invoke_id: u16,
    pm_store_handle: u16,
    u0: u32,
    u1: u32,
    entries: u16,
) -> Vec<u8> {
    let mut msg = Vec::with_capacity(34);
    write_be16(&mut msg, APDU_TYPE_PRESENTATION_APDU);
    write_be16(&mut msg, 30);
    write_be16(&mut msg, 28);
    write_be16(&mut msg, invoke_id);
    write_be16(&mut msg, DATA_APDU_RESPONSE_CONFIRMED_EVENT_REPORT);
    write_be16(&mut msg, 22);
    write_be16(&mut msg, pm_store_handle);
    write_be32(&mut msg, 0xFFFF_FFFF);
    write_be16(&mut msg, EVENT_TYPE_MDC_NOTI_SEGMENT_DATA);
    write_be16(&mut msg, 12);
    write_be32(&mut msg, u0);
    write_be32(&mut msg, u1);
    write_be16(&mut msg, entries);
    write_be16(&mut msg, 0x0080); // confirmed
    msg
}

fn build_release_request() -> Vec<u8> {
    let mut msg = Vec::with_capacity(6);
    write_be16(&mut msg, APDU_TYPE_ASSOCIATION_RELEASE_REQUEST);
    write_be16(&mut msg, 2);
    write_be16(&mut msg, 0);
    msg
}

fn find_object<'a>(
    buffer: &'a [u8],
    requested_class: u16,
) -> Result<(&'a [u8], u16, u16), AccuChekError> {
    require_len(buffer, 28, "configuration report")?;
    let count = u16_at(buffer, 24, "configuration object count")?;
    let mut offset = 28usize;

    for _ in 0..count {
        require_len(buffer, offset + 8, "configuration object header")?;
        let obj_class = u16_at(buffer, offset, "configuration object class")?;
        let obj_handle = u16_at(buffer, offset + 2, "configuration object handle")?;
        let attribute_count = u16_at(buffer, offset + 4, "configuration attribute count")?;
        let object_size = u16_at(buffer, offset + 6, "configuration object size")? as usize;
        offset += 8;
        require_len(buffer, offset + object_size, "configuration object payload")?;
        if obj_class == requested_class {
            return Ok((&buffer[offset..offset + object_size], attribute_count, obj_handle));
        }
        offset += object_size;
    }

    Err(protocol_error("configuration report", "requested object not found"))
}

fn find_attribute<'a>(
    buffer: &'a [u8],
    attribute_count: u16,
    requested_class: u16,
) -> Result<&'a [u8], AccuChekError> {
    let mut offset = 0usize;
    for _ in 0..attribute_count {
        require_len(buffer, offset + 4, "attribute header")?;
        let class = u16_at(buffer, offset, "attribute class")?;
        let size = u16_at(buffer, offset + 2, "attribute size")? as usize;
        offset += 4;
        require_len(buffer, offset + size, "attribute payload")?;
        if class == requested_class {
            return Ok(&buffer[offset..offset + size]);
        }
        offset += size;
    }
    Err(protocol_error("attribute list", format!("attribute {} not found", requested_class)))
}

fn mds_attribute<'a>(buffer: &'a [u8], requested_class: u16) -> Result<&'a [u8], AccuChekError> {
    require_len(buffer, 18, "MDS response")?;
    let attribute_count = u16_at(buffer, 14, "MDS attribute count")?;
    let payload_len = u16_at(buffer, 16, "MDS attribute list length")? as usize;
    require_len(buffer, 18 + payload_len, "MDS attribute list")?;
    find_attribute(&buffer[18..18 + payload_len], attribute_count, requested_class)
}

fn production_spec_entry<'a>(buffer: &'a [u8], requested_type: u16) -> Result<&'a [u8], AccuChekError> {
    require_len(buffer, 2, "production specification")?;
    let count = u16_at(buffer, 0, "production specification count")?;
    let mut offset = 0usize;
    for _ in 0..count {
        require_len(buffer, offset + 10, "production specification entry")?;
        let entry_type = u16_at(buffer, offset + 4, "production specification type")?;
        let len = u16_at(buffer, offset + 8, "production specification value length")? as usize;
        let start = offset + 10;
        require_len(buffer, start + len, "production specification value")?;
        if entry_type == requested_type {
            return Ok(&buffer[start..start + len]);
        }
        offset = start + len;
    }
    Err(protocol_error(
        "production specification",
        format!("entry type {} not found", requested_type),
    ))
}

fn decode_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_matches('\0')
        .trim()
        .to_string()
}

fn first_number(text: &str) -> Option<u16> {
    let digits: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn model_name(number: Option<u16>) -> String {
    let Some(number) = number else {
        return "Unknown Accu-Chek".to_string();
    };
    let name = match number {
        483 | 484 | 497 | 498 | 499 | 500 | 502 | 685 => "Aviva Connect",
        479 | 501 | 503 | 765 => "Performa Connect",
        921 | 922 | 923 | 925 | 926 | 929 | 930 | 932 => "Guide",
        958 | 959 | 960 | 961 | 963 | 964 | 965 => "Instant (single-button)",
        897 | 898 | 901 | 902 | 903 | 904 | 905 => "Guide Me",
        972 | 973 | 975 | 976 | 977 | 978 | 979 | 980 => "Instant (two-button)",
        966 | 967 | 968 | 969 | 970 | 971 => "Instant S (single-button)",
        982 => "ReliOn Platinum",
        _ => return format!("Unknown model {}", number),
    };
    name.to_string()
}

fn bcd(byte: u8, context: &str) -> Result<u32, AccuChekError> {
    let hi = (byte >> 4) & 0x0f;
    let lo = byte & 0x0f;
    if hi > 9 || lo > 9 {
        return Err(protocol_error(context, format!("invalid BCD byte 0x{:02x}", byte)));
    }
    Ok((hi as u32) * 10 + lo as u32)
}

fn decode_datetime(bytes: &[u8], context: &str) -> Result<(String, i64), AccuChekError> {
    require_len(bytes, 6, context)?;
    let cc = bcd(bytes[0], context)?;
    let yy = bcd(bytes[1], context)?;
    let month = bcd(bytes[2], context)?;
    let day = bcd(bytes[3], context)?;
    let hour = bcd(bytes[4], context)?;
    let minute = bcd(bytes[5], context)?;
    let year = (cc * 100 + yy) as i32;

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| protocol_error(context, "invalid calendar date"))?;
    let dt = date
        .and_hms_opt(hour, minute, 0)
        .ok_or_else(|| protocol_error(context, "invalid wall-clock time"))?;

    // Compatibility sort key only. The meter supplies local wall-clock time, not a UTC offset.
    let local_sort_epoch = dt.and_utc().timestamp();
    Ok((dt.format("%Y-%m-%dT%H:%M:%S").to_string(), local_sort_epoch))
}

fn normalize_glucose(raw: u16) -> (u16, ReadingRange) {
    match raw {
        0x07FE => (601, ReadingRange::High),
        0x0802 => (9, ReadingRange::Low),
        value => (value, ReadingRange::Normal),
    }
}

fn parse_page(
    buffer: &[u8],
    device_key: &str,
    next_id: &mut usize,
) -> Result<Vec<GlucoseReading>, AccuChekError> {
    require_len(buffer, 33, "segment data")?;
    let entries = u16_at(buffer, 30, "segment entry count")? as usize;
    let mut offset = 30usize;
    let mut result = Vec::with_capacity(entries);

    for index in 0..entries {
        require_len(buffer, offset + 18, "glucose record")?;
        let (timestamp, epoch) = decode_datetime(
            &buffer[offset + 6..offset + 12],
            &format!("glucose record {} timestamp", index),
        )?;
        let raw_value = u16_at(buffer, offset + 14, "glucose value")?;
        let status = u16_at(buffer, offset + 16, "glucose status")?;
        let (mg_dl, range_state) = normalize_glucose(raw_value);
        let mmol_l = mg_dl as f64 / 18.0;

        result.push(GlucoseReading {
            id: *next_id,
            epoch,
            timestamp,
            mg_dl,
            mmol_l,
            raw_value,
            status,
            range_state,
            device_key: device_key.to_string(),
        });
        *next_id += 1;
        offset += 12;
    }

    Ok(result)
}

fn metadata_from_mds(accu_chek: &AccuChekDevice, buffer: &[u8]) -> DeviceMetadata {
    let model_text = mds_attribute(buffer, MDC_ATTR_ID_MODEL).ok().map(decode_text);
    let model_number = model_text.as_deref().and_then(first_number);

    // Preserve the production-spec serial verbatim (apart from padding/whitespace). Do not
    // coerce it to an integer: serials are identifiers, can exceed u16, and may contain letters.
    let serial_number = mds_attribute(buffer, MDC_ATTR_ID_PROD_SPECN)
        .ok()
        .and_then(|spec| production_spec_entry(spec, 1).ok()) // 1 = serial-number
        .map(decode_text)
        .filter(|serial| !serial.is_empty());

    let meter_time = mds_attribute(buffer, MDC_ATTR_TIME_ABS)
        .ok()
        .and_then(|value| decode_datetime(value, "meter time").ok())
        .map(|(timestamp, _)| timestamp);

    let stable_serial = serial_number.clone().or_else(|| accu_chek.usb_serial.clone());
    let device_key = match stable_serial.as_deref() {
        Some(serial) => format!(
            "roche-{:04x}-{:04x}-{}",
            accu_chek.vendor_id, accu_chek.product_id, serial
        ),
        None => format!(
            "roche-{:04x}-{:04x}-unknown",
            accu_chek.vendor_id, accu_chek.product_id
        ),
    };

    DeviceMetadata {
        vendor_id: accu_chek.vendor_id,
        product_id: accu_chek.product_id,
        manufacturer: accu_chek.vendor.clone(),
        usb_product: accu_chek.product.clone(),
        usb_serial: accu_chek.usb_serial.clone(),
        model_number,
        model_name: model_name(model_number),
        serial_number,
        meter_time,
        device_key,
    }
}
