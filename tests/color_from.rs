use draw::*;

#[test]
fn color_from_test() {
    {
        let expected = Rgba([0x12, 0x34, 0x56, 0x78]);
        let found = Rgba::from_rgba_u32(0x12_34_56_78);
        assert!(expected == found);
    }
    {
        let expected = Rgba([0x12, 0x34, 0x56, 0x78]);
        let found = Rgba::from_argb_u32(0x78_12_34_56);
        assert!(expected == found);
    }
    {
        let expected = Rgba([0x12, 0x34, 0x56, 0x78]);
        let found = Rgba::from_le_u32(0x78_56_34_12);
        assert!(expected == found);
    }
}

#[test]
fn color_parsing() {
    {
        let expected = Rgba([0x12, 0x34, 0x56, 0x78]);
        let found: Rgba = "0x87123456".parse().unwrap();
        assert!(expected == found);
    }
}
