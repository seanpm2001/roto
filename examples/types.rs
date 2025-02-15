use roto::types::builtin::BuiltinTypeValue;
use roto::types::collections::{ElementTypeValue, List, Record};
use roto::types::typedef::TypeDef;
use roto::types::typevalue::TypeValue;

use routecore::bgp::communities::HumanReadableCommunity as Community;
use inetnum::asn::Asn;

fn main() {
    // let count = RotoType::create_primitive_var(
    //     RotoType::Asn,
    //     RotoPrimitiveType::Asn(Asn::from_u32(1)),
    // )
    // .unwrap();

    let count =
        TypeValue::from(1_u32);

    let count2 = TypeValue::from(
        inetnum::addr::Prefix::new("193.0.0.0".parse().unwrap(), 24)
            .unwrap(),
    );

    let ip_address = TypeValue::from(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(193, 0, 0, 23)),
    );

    let as_path = TypeValue::from(
        BuiltinTypeValue::AsPath(
            vec![inetnum::asn::Asn::from_u32(1)].into()
        )
    );

    let asn = TypeValue::from(
        Asn::from_u32(211321)
    );
    println!("{:?}", asn);

    let comms =
        TypeValue::List(List::new(vec![ElementTypeValue::Primitive(
            Community::from([127, 12, 13, 12]).into())
        ]));

    let my_comms_type =
        TypeDef::List(Box::new(TypeDef::List(Box::new(TypeDef::Community))));

    let my_nested_rec_type =
        TypeDef::new_record_type(vec![("counter", Box::new(TypeDef::U32))])
            .unwrap();

    let my_nested_rec_instance = Record::create_instance_with_ordered_fields(
        &my_nested_rec_type,
        vec![(
            "counter",
            TypeValue::Builtin(BuiltinTypeValue::U32(1)),
        )],
    )
    .unwrap();

    let my_rec_type = TypeDef::new_record_type(vec![
        ("count", Box::new(TypeDef::U32)),
        ("count2", Box::new(TypeDef::Prefix)),
        ("ip_address", Box::new(TypeDef::IpAddr)),
        ("asn", Box::new(TypeDef::Asn)),
        ("as_path", Box::new(TypeDef::AsPath)),
        ("communities", Box::new(my_comms_type)),
        ("record", Box::new(my_nested_rec_type)),
    ])
    .unwrap();

    let my_record = Record::create_instance_with_ordered_fields(
        &my_rec_type,
        vec![
            ("count", count),
            ("count2", count2),
            ("ip_address", ip_address),
            ("asn", asn),
            ("as_path", as_path),
            ("communities", comms),
            ("record", TypeValue::Record(my_nested_rec_instance)),
        ],
    )
    .unwrap();

    println!("{:?}", my_record);
    println!("{:?}", my_record.get_value_for_field("as_path"));
}
