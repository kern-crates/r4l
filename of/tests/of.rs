static BST_DTB_DATA: &[u8] = include_bytes!("./bsta1000b-fada-bus.dtb");

fn setup() {
    unsafe {
        of_fdt::init_fdt_ptr(BST_DTB_DATA.as_ptr());
    }
}

#[test]
fn test_module() {
    setup();
    let model = of_fdt::machin_name();
    assert_eq!(model, "BST A1000B FAD-A");
}

#[test]
fn test_find_compatible() {
    const CONSOLE_COMPATIABLE: &'static [&'static str] = &["snps,dw-apb-uart"];
    const CONSOLE_COUNT: usize = 4;
    setup();
    let console_node = of_fdt::find_compatible_node(CONSOLE_COMPATIABLE);
    assert_eq!(console_node.count(), CONSOLE_COUNT);
}


#[test]
fn test_find_i2c_compatible() {
    const CONSOLE_COMPATIABLE: &'static [&'static str] = &["snps,designware-i2c"];
    setup();
    let mut i2c_bus_node = of_fdt::find_compatible_node(CONSOLE_COMPATIABLE);

    let i2c0 = i2c_bus_node.next().unwrap();
    let i2c1 = i2c_bus_node.next().unwrap();
    let i2c2 = i2c_bus_node.next().unwrap();
    assert_eq!(i2c0.name, "i2c@20000000");
    assert_eq!(i2c1.name, "i2c@20001000");
    assert_eq!(i2c2.name, "i2c@20002000");
    assert!(i2c0.children().next().is_none());
    let i2c2_child = i2c2.children().next().unwrap();
    assert_eq!(i2c2_child.name, "max96712@29");
}

#[test]
fn test_pcsi() {
    setup();
    let of_pcsi = of_fdt::pcsi();
    assert!(of_pcsi.is_some());
    let of_pcsi = of_pcsi.unwrap();
    assert_eq!(of_pcsi.method(), "smc");
    assert_eq!(of_pcsi.cpu_on().unwrap(), 0xC4000003);
    assert_eq!(of_pcsi.cpu_off().unwrap(), 0x84000002);
    assert_eq!(of_pcsi.cpu_suspend().unwrap(), 0xC4000001);
}

#[test]
fn test_platform() {
    const OF_DEFAULT_BUS_MATCH_TABLE: [&'static [&'static str]; 4] = [
        &["simple-bus"],
        &["simple-mfd"],
        &["simple-isa"],
        &["arm,amba-bus"],
    ];
    setup();
    for b in OF_DEFAULT_BUS_MATCH_TABLE {
        let bus_nodes = of_fdt::find_compatible_node(b);
        if b[0].eq("simple-bus") {
            assert_eq!(bus_nodes.count(), 1);
        } else {
            assert_eq!(bus_nodes.count(), 0);
        }
    }
}

#[test]
fn test_irqcontroler() {
    const I2C_COMPATIABLE: &'static [&'static str] = &["snps,designware-i2c"];
    setup();
    let i2c_node = of_fdt::find_compatible_node(I2C_COMPATIABLE).next().unwrap();
    let irq_controler = i2c_node.interrupt_parent().unwrap();
    assert_eq!("arm,gic-400", irq_controler.compatible().unwrap().first());
    assert_eq!(3, irq_controler.interrupt_cells().unwrap());
    let mut res: [u32; 3] = [0; 3];
    for i in 0..irq_controler.interrupt_cells().unwrap() {
        res[i] = of_fdt::of_property_read_u32(i2c_node, "interrupts", i).unwrap();
    }
    assert_eq!([0, 0xcf, 0x04], res);
}

#[test]
fn test_phandle_arg() {
    const I2C_COMPATIABLE: &'static [&'static str] = &["snps,designware-i2c"];
    setup();

    let i2c_node = of_fdt::find_compatible_node(I2C_COMPATIABLE).next().unwrap();
    let phandle_arg = of_fdt::of_parse_phandle_with_args(i2c_node, "clocks", Some("#clock-cells"), 0)
        .expect("i2c no clocks");
    assert_eq!(phandle_arg.args_count, 1);
    assert_eq!(phandle_arg.args[0], 75); // LSP0_PCLK
    let phandle_arg = of_fdt::of_parse_phandle_with_args(i2c_node, "clocks", Some("#clock-cells"), 1)
        .expect("i2c no clocks");
    assert_eq!(phandle_arg.args[0], 73); // LSP0_WCLK
}
