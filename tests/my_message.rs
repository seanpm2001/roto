use log::trace;

use roto::blocks::Scope;
use roto::types::builtin::{NlriStatus, PeerId, PeerRibType, Provenance, RouteContext};
use roto::pipeline;
use roto::types::collections::{ElementTypeValue, List, Record};
use roto::types::typedef::TypeDef;
use roto::types::typevalue::TypeValue;
use roto::vm::{self, VmResult};

use routecore::bgp::communities::HumanReadableCommunity as Community;
use inetnum::asn::Asn;

mod common;

fn test_data(
    name: Scope,
    source_code: &'static str,
) -> Result<VmResult, Box<dyn std::error::Error>> {
    println!("Evaluate filter-map {}...", name);

    let filter_map_arguments =
        vec![("my_asn", TypeValue::from(Asn::from(65534_u32)))];

    let rotolo =
        pipeline::run_test(source_code, Some((&name, filter_map_arguments)))?;
    let roto_pack = rotolo.retrieve_pack_as_refs(&name)?;

    let _count: TypeValue = 1_u32.into();
    let prefix: TypeValue =
        inetnum::addr::Prefix::new("193.0.0.0".parse().unwrap(), 24)?
            .into();
    let next_hop: TypeValue =
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(193, 0, 0, 23)).into();
    let as_path = vec![Asn::from_u32(1)].into();
    let asn: TypeValue = Asn::from_u32(211321).into();

    println!("{:?}", asn);

    let comms =
        TypeValue::List(List::new(vec![ElementTypeValue::Primitive(
            Community::from([127, 12, 13, 12]).into(),
        )]));

    let my_comms_type = (&comms).into();

    let my_nested_rec_type =
        TypeDef::new_record_type(vec![("counter", Box::new(TypeDef::U32))])
            .unwrap();

    let _my_nested_rec_instance =
        Record::create_instance_with_ordered_fields(
            &my_nested_rec_type,
            vec![("counter", 1_u32.into())],
        )
        .unwrap();

    let my_rec_type = TypeDef::new_record_type(vec![
        ("prefix", Box::new(TypeDef::Prefix)),
        ("as-path", Box::new(TypeDef::AsPath)),
        ("origin", Box::new(TypeDef::Asn)),
        ("next-hop", Box::new(TypeDef::IpAddr)),
        ("med", Box::new(TypeDef::U32)),
        ("local-pref", Box::new(TypeDef::U32)),
        ("community", Box::new(my_comms_type)),
    ])
    .unwrap();

    let my_payload = Record::create_instance_with_ordered_fields(
        &my_rec_type,
        vec![
            ("prefix", prefix),
            ("as-path", as_path),
            ("origin", asn),
            ("next-hop", next_hop),
            ("med", 80_u32.into()),
            ("local-pref", 20_u32.into()),
            ("community", comms),
        ],
    )
    .unwrap();

    let mem = &mut vm::LinearMemory::uninit();

    println!("Used Arguments");
    println!("{:#?}", &roto_pack.arguments);
    println!("Used Data Sources");
    println!("{:#?}", &roto_pack.data_sources);

    for mb in roto_pack.get_mir().iter() {
        println!("{}", mb);
    }

    let my_payload = TypeValue::Record(my_payload);
    assert!(roto_pack.check_rx_payload_type(&my_payload));

    let ds_ref = roto_pack.data_sources;

    
    let peer_ip = "192.0.2.0".parse().unwrap();

    let provenance = Provenance {
        timestamp: chrono::Utc::now(),
        connection_id: "192.0.2.0:178".parse().unwrap(),
        peer_id: PeerId { addr: peer_ip, asn: Asn::from(65534) },
        peer_bgp_id: [0; 4].into(),
        peer_distuingisher: [0; 8],
        peer_rib_type: PeerRibType::OutPost,
    };

    let context = RouteContext::new(None, NlriStatus::InConvergence, provenance);

    println!("Start vm...");
    let mut vm = vm::VmBuilder::new()
        // .with_arguments(args)
        .with_context(context)
        .with_data_sources(ds_ref)
        .with_mir_code(roto_pack.mir)
        .build()?;

    let res = vm.exec(my_payload, None::<Record>, None, mem).unwrap();

    println!("\nRESULT");
    println!("action: {}", res.accept_reject);
    println!("rx    : {:?}", res.rx);
    println!("tx    : {:?}", res.tx);
    println!("stream: {:?}", res.output_stream_queue);

    Ok(res)
}

#[test]
fn test_filter_map_message_1() {
    common::init();

    test_data(
        Scope::FilterMap("my-message-filter-map-1".into()),
        r#"
        filter-map my-message-filter-map-1 with my_asn: Asn {
            define {
                // specify the types of that this filter receives
                // and sends.
                // rx_tx route: StreamRoute;
                rx route: MyPayload;
                tx out: Route;
            }

            term rov-valid for route: Route {
                match {
                    route.as-path.origin() == my_asn;
                }
            }
            
            action send-message {
                mqtt.send({ 
                    message: String.format("🤭 I encountered ", my_asn),
                    asn: my_asn
                });
            }

            apply {
                filter match rov-valid not matching {  
                    send-message;
                };
            }
        }

        output-stream mqtt contains Message {
            asn: Asn,
            message: String
        }

        type MyPayload {
            prefix: Prefix,
            as-path: AsPath,
            origin: Asn,
            next-hop: IpAddress,
            med: U32,
            local-pref: U32,
            community: [Community]
        }
        "#,
    )
    .unwrap();
}

#[test]
fn test_filter_map_message_2() {
    common::init();

    let res = test_data(
        Scope::FilterMap("my-message-filter-map-2".into()),
        r#"
        filter-map my-message-filter-map-2 with my_asn: Asn {
            define {
                // specify the types of that this filter receives
                // and sends.
                // rx_tx route: StreamRoute;
                rx route: MyPayload;
                tx out: Route;
            }

            term rov-valid for route: Route {
                match {
                    route.as-path.origin() == my_asn;
                }
            }
            
            action send-message {
                mqtt.send({ 
                    my_asn: my_asn,
                    message: String.format("🤭 I, the messager, saw {} in a BGP update.", my_asn)
                });
            }

            apply {
                filter match rov-valid not matching {  
                    send-message;
                };
            }
        }

        output-stream mqtt contains Message {
            asn: Asn,
            message: String
        }

        type MyPayload {
            prefix: Prefix,
            as-path: AsPath,
            origin: Asn,
            next-hop: IpAddress,
            med: U32,
            local-pref: U32,
            community: [Community]
        }
        "#,
    );

    res.unwrap_err();
}

#[test]
fn test_filter_map_message_3() {
    common::init();
    test_data(
        Scope::Filter("my-message-filter-map-3".into()),
        r#"
        filter my-message-filter-map-3 with my_asn: Asn {
            define {
                // specify the types of that this filter receives
                // and sends.
                // rx_tx route: StreamRoute;
                rx route: MyPayload;
            }

            term rov-valid for route: Route {
                match {
                    route.as-path.origin() == my_asn;
                }
            }
            
            action send-message {
                mqtt.send({ 
                    asn: my_asn,
                    message: String.format("🤭 I, the messager, saw {} in a BGP update.", my_asn)
                });
            }

            apply {
                filter match rov-valid not matching {  
                    send-message;
                };
            }
        }

        output-stream mqtt contains Message {
            asn: Asn,
            message: String
        }

        type MyPayload {
            prefix: Prefix,
            as-path: AsPath,
            origin: Asn,
            next-hop: IpAddress,
            med: U32,
            local-pref: U32,
            community: [Community]
        }
        "#,
    ).unwrap();
}

#[test]
fn test_filter_map_message_4() {
    common::init();

    let res = test_data(
        Scope::Filter("my-message-filter-map-2".into()),
        r#"
        filter my-message-filter-map-2 with my_asn: Asn {
            define {
                // specify the types of that this filter receives
                // and sends.
                // rx_tx route: StreamRoute;
                rx route: MyPayload;
                tx route: Route;
            }

            term rov-valid for route: Route {
                match {
                    route.as-path.origin() == my_asn;
                }
            }
            
            action send-message {
                mqtt.send({ 
                    asn: my_asn,
                    message: String.format("🤭 I, the messager, saw {} in a BGP update.", my_asn)
                });
            }

            apply {
                filter match rov-valid not matching {  
                    send-message;
                };
            }
        }

        output-stream mqtt contains Message {
            asn: Asn,
            message: String
        }

        type MyPayload {
            prefix: Prefix,
            as-path: AsPath,
            origin: Asn,
            next-hop: IpAddress,
            med: U32,
            local-pref: U32,
            community: [Community]
        }
    "#,
    );

    res.unwrap_err();
}

#[test]
fn test_filter_map_message_5() {
    common::init();
    let res = test_data(
        Scope::Filter("my-message-filter-map-5".into()),
        r#"
        filter my-message-filter-map-5 with my_asn: Asn {
            define {
                // specify the types of that this filter receives
                // and sends.
                // rx_tx route: StreamRoute;
                rx route: MyPayload;
            }

            term rov-valid for route: Route {
                match {
                    route.as-path.origin() == my_asn;
                }
            }
            
            action send-message {
                mqtt.send({ 
                    name: "My ASN",
                    topic: "My Asn was Seen!",
                    asn: my_asn,
                    message: String.format("🤭 I, the messager, saw {} in a BGP update.", my_asn)
                });
            }

            apply {
                filter match rov-valid not matching {  
                    send-message;
                };
            }
        }

        output-stream mqtt contains Message {
            asn: Asn,
            message: String,
            name: String,
            topic: String
        }

        type MyPayload {
            prefix: Prefix,
            as-path: AsPath,
            origin: Asn,
            next-hop: IpAddress,
            med: U32,
            local-pref: U32,
            community: [Community]
        }
        "#,
    ).unwrap();

    assert_eq!(res.output_stream_queue.len(), 1);
    assert_eq!(res.output_stream_queue[0].get_name(), "My ASN");
    assert_eq!(res.output_stream_queue[0].get_topic(), "My Asn was Seen!");
}

#[test]
fn test_filter_map_message_6() {
    common::init();
    let res = test_data(
        Scope::Filter("my-message-filter-map-6".into()),
        r#"
        filter my-message-filter-map-6 with my_asn: Asn {
            define {
                // specify the types of that this filter receives
                // and sends.
                // rx_tx route: StreamRoute;
                rx route: MyPayload;
            }

            term rov-valid for route: Route {
                match {
                    route.as-path.origin() == my_asn;
                }
            }
            
            action send-message {
                mqtt.send({ 
                    name: "My ASN",
                    topic: "My Asn was Seen!",
                    asn: my_asn,
                    message: String.format("🤭 I, the messager, saw {} in a BGP update.", my_asn)
                });
            }

            apply {
                filter match rov-valid not matching {  
                    send-message;
                };
            }
        }

        output-stream mqtt contains Message {
            name: String,
            topic: String,
            asn: Asn,
            message: String
        }

        type MyPayload {
            prefix: Prefix,
            as-path: AsPath,
            origin: Asn,
            next-hop: IpAddress,
            med: U32,
            local-pref: U32,
            community: [Community]
        }
        "#,
    ).unwrap();

    for m in res.output_stream_queue.iter() {
        trace!("MESSAGE {:?}", m);
    }
    assert_eq!(res.output_stream_queue.len(), 1);
    assert_eq!(res.output_stream_queue[0].get_name(), "My ASN");
}
