use roto::types::{
    AsPath, Asn, Community, CommunityType, ElementType, List, Prefix, Record,
    RotoPrimitiveType, RotoType, RotoTypeValue, U32,
};

fn main() {
    // let count = RotoType::create_primitive_var(
    //     RotoType::Asn,
    //     RotoPrimitiveType::Asn(Asn::from_u32(1)),
    // )
    // .unwrap();

    let count =
        RotoPrimitiveType::create_instance(RotoType::U32, 1_u32).unwrap();
    let count2 =
        RotoTypeValue::Primitive(RotoPrimitiveType::Prefix(Prefix::new(
            routecore::addr::Prefix::new("193.0.0.0".parse().unwrap(), 24)
                .unwrap(),
        )));

    let ip_address = RotoPrimitiveType::create_instance(
        RotoType::IpAddress,
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(193, 0, 0, 23)),
    )
    .unwrap();

    let as_path = RotoPrimitiveType::create_instance(
        RotoType::AsPath,
        RotoPrimitiveType::AsPath(AsPath::new(vec![Asn::from_u32(1)])),
    )
    .unwrap();

    let asn = RotoPrimitiveType::create_instance(
        RotoType::Asn,
        Asn::from_u32(211321),
    )
    .unwrap();
    println!("{:?}", asn);

    let comms = RotoTypeValue::List(List::new(vec![ElementType::Primitive(
        RotoPrimitiveType::Community(Community::new(CommunityType::Normal)),
    )]));

    let my_comms_type = RotoType::List(Box::new(RotoType::List(Box::new(
        RotoType::Community,
    ))));

    let my_nested_rec_type =
        RotoType::new_record_type(vec![("counter", RotoType::U32)]).unwrap();

    let my_nested_rec_instance = Record::create_instance(
        &my_nested_rec_type,
        vec![(
            "counter",
            RotoTypeValue::Primitive(RotoPrimitiveType::U32(U32::new(1))),
        )],
    )
    .unwrap();

    let my_rec_type = RotoType::new_record_type(vec![
        ("count", RotoType::U32),
        ("count2", RotoType::Prefix),
        ("ip_address", RotoType::IpAddress),
        ("asn", RotoType::Asn),
        ("as_path", RotoType::AsPath),
        ("communities", my_comms_type),
        ("record", my_nested_rec_type),
    ])
    .unwrap();

    let my_record = Record::create_instance(
        &my_rec_type,
        vec![
            ("count", count),
            ("count2", count2),
            ("ip_address", ip_address),
            ("asn", asn),
            ("as_path", as_path),
            ("communities", comms),
            ("record", RotoTypeValue::Record(my_nested_rec_instance)),
        ],
    )
    .unwrap();

    println!("{:?}", my_record);
    println!("{:?}", my_record.get_value_for_field("as_path"));
}
