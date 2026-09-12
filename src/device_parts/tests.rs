#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn association_response_matches_ieee11073_shape() {
        let msg = build_association_response();
        assert_eq!(msg.len(), 48);
        assert_eq!(u16::from_be_bytes([msg[0], msg[1]]), APDU_TYPE_ASSOCIATION_RESPONSE);
        assert_eq!(u16::from_be_bytes([msg[2], msg[3]]), 44);
        assert_eq!(&msg[30..38], &[0x12, 0x34, 0x56, 0x78, 0x87, 0x65, 0x43, 0x21]);
    }

    #[test]
    fn fixed_message_lengths_are_exact() {
        assert_eq!(build_config_response(1).len(), 26);
        assert_eq!(build_mds_request(1).len(), 18);
        assert_eq!(build_segment_info_request(1, 2).len(), 24);
        assert_eq!(build_data_transfer_request(1, 2).len(), 20);
        assert_eq!(build_data_confirmation(1, 2, 3, 4, 5).len(), 34);
        assert_eq!(build_release_request().len(), 6);
    }

    #[test]
    fn bcd_validation_rejects_non_decimal_nibbles() {
        assert_eq!(bcd(0x42, "test").unwrap(), 42);
        assert!(bcd(0xFA, "test").is_err());
    }

    #[test]
    fn range_sentinels_are_preserved_and_normalized() {
        assert_eq!(normalize_glucose(0x07FE), (601, ReadingRange::High));
        assert_eq!(normalize_glucose(0x0802), (9, ReadingRange::Low));
        assert_eq!(normalize_glucose(123), (123, ReadingRange::Normal));
    }

    #[test]
    fn nonzero_status_is_retained_not_dropped() {
        let mut page = vec![0u8; 48];
        page[30..32].copy_from_slice(&1u16.to_be_bytes());
        page[36..42].copy_from_slice(&[0x20, 0x26, 0x09, 0x13, 0x14, 0x05]);
        page[44..46].copy_from_slice(&123u16.to_be_bytes());
        page[46..48].copy_from_slice(&0x0040u16.to_be_bytes());

        let mut next_id = 0;
        let readings = parse_page(&page, "device", &mut next_id).unwrap();
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].mg_dl, 123);
        assert_eq!(readings[0].raw_value, 123);
        assert_eq!(readings[0].status, 0x0040);
        assert_eq!(readings[0].timestamp, "2026-09-13T14:05:00");
    }

    #[test]
    fn truncated_page_fails_instead_of_panicking() {
        let mut next_id = 0;
        assert!(parse_page(&[0u8; 10], "device", &mut next_id).is_err());
    }

    #[test]
    fn tidepool_model_mapping_covers_instant_families() {
        assert_eq!(model_name(Some(958)), "Instant (single-button)");
        assert_eq!(model_name(Some(972)), "Instant (two-button)");
        assert_eq!(model_name(Some(966)), "Instant S (single-button)");
    }

    #[test]
    fn roche_vendor_constant_is_expected() {
        assert_eq!(ROCHE_VENDOR_ID, 0x173a);
    }

    #[test]
    fn duplicate_collision_key_components_can_be_counted_by_caller() {
        let mut counts = std::collections::HashMap::<(&str, u16, u16), usize>::new();
        *counts.entry(("2026-09-13T14:05:00", 123, 0)).or_default() += 1;
        *counts.entry(("2026-09-13T14:05:00", 123, 0)).or_default() += 1;
        assert_eq!(counts.values().next().copied(), Some(2));
    }
}
