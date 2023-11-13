use log::trace;
use roto::ast::AcceptReject;
use roto::compiler::Compiler;

use roto::blocks::Scope::{self, Filter};
use roto::types::builtin::Prefix;
use roto::types::builtin::{
    RawRouteWithDeltas, RotondaId, RouteStatus, UpdateMessage,
};
use roto::types::collections::Record;
use roto::types::typevalue::TypeValue;
use roto::vm::{self, VmResult};
use routecore::bgp::message::nlri::{Nlri, BasicNlri};
use routecore::bgp::message::SessionConfig;

mod common;

fn test_data(
    name: Scope,
    source_code: &str,
) -> Result<(VmResult, TypeValue, TypeValue), Box<dyn std::error::Error>> {
    trace!("Evaluate filter-map {}...", name);

    let c = Compiler::new();
    let roto_packs = c.build_from_compiler(source_code)?;

    let roto_pack = roto_packs.retrieve_pack_as_refs(&name)?;

    // BGP UPDATE message containing MP_REACH_NLRI path attribute,
    // comprising 5 IPv6 NLRIs
    let buf = bytes::Bytes::from(vec![
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x88, 0x02, 0x00, 0x00, 0x00,
        0x71, 0x80, 0x0e, 0x5a, 0x00, 0x02, 0x01, 0x20, 0xfc, 0x00, 0x00,
        0x10, 0x00, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x10, 0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0xfc, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x10, 0x40, 0x20, 0x01, 0x0d, 0xb8, 0xff, 0xff, 0x00,
        0x00, 0x40, 0x20, 0x01, 0x0d, 0xb8, 0xff, 0xff, 0x00, 0x01, 0x40,
        0x20, 0x01, 0x0d, 0xb8, 0xff, 0xff, 0x00, 0x02, 0x40, 0x20, 0x01,
        0x0d, 0xb8, 0xff, 0xff, 0x00, 0x03, 0x40, 0x01, 0x01, 0x00, 0x40,
        0x02, 0x06, 0x02, 0x01, 0x00, 0x00, 0x00, 0xc8, 0x80, 0x04, 0x04,
        0x00, 0x00, 0x00, 0x00,
    ]);

    let msg_id = (RotondaId(0), 0);
    let update: UpdateMessage =
        UpdateMessage::new(buf.clone(), SessionConfig::modern());

    let first_prefix: Prefix = update
    .0
    .announcements()
    .into_iter()
    .next()
    .and_then(|mut p| if let Some(Ok(Nlri::Unicast(BasicNlri { prefix, .. }))) = p.next() { Some(Prefix::from(prefix)) } else { None }).unwrap();
    let peer_ip = "fe80::1".parse().unwrap();

    let payload: RawRouteWithDeltas = RawRouteWithDeltas::new_with_message(
        msg_id,
        first_prefix,
        update,
        RouteStatus::InConvergence,
    ).with_peer_ip(peer_ip);

    trace!("prefix in route {:?}", payload.prefix);
    trace!("peer_ip {:?}", payload.peer_ip());

    let mem = &mut vm::LinearMemory::uninit();

    println!("Used Arguments");
    println!("{:#?}", &roto_pack.arguments);
    println!("Used Data Sources");
    println!("{:#?}", &roto_pack.data_sources);

    for mb in roto_pack.get_mir().iter() {
        println!("{}", mb);
    }

    let mut vm = vm::VmBuilder::new()
        .with_data_sources(roto_pack.data_sources)
        .with_mir_code(roto_pack.mir)
        .build()?;

    let res = vm.exec(payload, None::<Record>, None, mem).unwrap();

    println!("\nRESULT");
    println!("action: {}", res.accept_reject);
    println!("rx    : {:?}", res.rx);
    println!("tx    : {:?}", res.tx);

    Ok((res, first_prefix.into(), peer_ip.into()))
}

#[test]
fn test_routes_1() {
    common::init();
    let src = r#"
        filter rib-in-pre-filter {
            define {
                rx route: Route;
            }
        
            term always {
                match {
                    1 == 1;
                }
            }
        
            action send_msg {
                // An untyped anonymous record
                mqtt.send(
                    {
                        prefix: route.prefix,
                        peer_ip: route.peer_ip
                    }
                );
            }
        
            apply {
                filter match always matching {
                    send_msg;
                    return accept;
                };
                reject;
            }
        }
        
        output-stream mqtt contains Message {
            prefix: Prefix,
            peer_ip: IpAddress
        }
    "#;

    let test_run = test_data(Filter("rib-in-pre-filter".into()), src);

    let (VmResult { accept_reject, output_stream_queue, .. }, prefix, peer_ip) = test_run.unwrap();
    let output_record = output_stream_queue[0].get_record();
    trace!("{:#?}", output_record);
    assert_eq!(output_stream_queue.len(), 1);

    // The outputted record is an ordered_record, meaning it is sorted
    // alphabetically on the key, so [0] is "peer_ip" and [1] is "prefix"
    assert_eq!(output_record.get_field_by_index(vec![1].into()).unwrap(), &prefix);
    assert_eq!(output_record.get_field_by_index(vec![0].into()).unwrap(), &peer_ip);
    trace!("{:#?}", output_stream_queue);
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_routes_2() {
    common::init();
    let src = r#"
        filter rib-in-pre-filter {
            define {
                rx route: Route;
            }
        
            term always {
                match {
                    1 == 1;
                }
            }
        
            // A typed anonymous record
            action send_msg {
                mqtt.send(
                    Message {
                        prefix: route.prefix,
                        peer_ip: route.peer_ip
                    }
                );
            }
        
            apply {
                filter match always matching {
                    send_msg;
                    return accept;
                };
                reject;
            }
        }
        
        output-stream mqtt contains Message {
            prefix: Prefix,
            peer_ip: IpAddress
        }
    "#;

    let test_run = test_data(Filter("rib-in-pre-filter".into()), src);

    let (VmResult { accept_reject, output_stream_queue, .. }, prefix, peer_ip) = test_run.unwrap();
    let output_record = output_stream_queue[0].get_record();
    trace!("{:#?}", output_record);
    assert_eq!(output_stream_queue.len(), 1);

    // The outputted record is an ordered_record, meaning it is sorted
    // alphabetically on the key, so [0] is "peer_ip" and [1] is "prefix"
    assert_eq!(output_record.get_field_by_index(vec![1].into()).unwrap(), &prefix);
    assert_eq!(output_record.get_field_by_index(vec![0].into()).unwrap(), &peer_ip);
    trace!("{:#?}", output_stream_queue);
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_routes_3() {
    common::init();
    let src = r#"
        filter rib-in-pre-filter {
            define {
                rx route: Route;

                msg = Message {
                    prefix: route.prefix,
                    peer_ip: route.peer_ip
                };
            }
        
            term always {
                match {
                    msg.prefix.exists();
                }
            }
        
            // A typed named record
            action send_msg {
                mqtt.send(msg);
            }
        
            apply {
                filter match always matching {
                    send_msg;
                    return accept;
                };
                reject;
            }
        }
        
        output-stream mqtt contains Message {
            prefix: Prefix,
            peer_ip: IpAddress
        }
    "#;

    let test_run = test_data(Filter("rib-in-pre-filter".into()), src);

    let (VmResult { accept_reject, output_stream_queue, .. }, prefix, peer_ip) = test_run.unwrap();
    let output_record = output_stream_queue[0].get_record();
    trace!("{:#?}", output_record);
    assert_eq!(output_stream_queue.len(), 1);

    // The outputted record is an ordered_record, meaning it is sorted
    // alphabetically on the key, so [0] is "peer_ip" and [1] is "prefix"
    assert_eq!(output_record.get_field_by_index(vec![1].into()).unwrap(), &prefix);
    assert_eq!(output_record.get_field_by_index(vec![0].into()).unwrap(), &peer_ip);
    trace!("{:#?}", output_stream_queue);
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_routes_4() {
    common::init();
    let src = r#"
        filter rib-in-pre-filter {
            define {
                rx route: Route;

                pfx = route.prefix;
                peer = route.peer_ip;
            }
        
            term always {
                match {
                    1 == 1;
                }
            }
        
            // A typed named record
            action send_msg {
                mqtt.send({
                    prefix: pfx,
                    peer_ip: peer
                });
            }
        
            apply {
                filter match always matching {
                    send_msg;
                    return accept;
                };
                reject;
            }
        }
        
        output-stream mqtt contains Message {
            prefix: Prefix,
            peer_ip: IpAddress
        }
    "#;

    let test_run = test_data(Filter("rib-in-pre-filter".into()), src);

    let (VmResult { accept_reject, output_stream_queue, .. }, prefix, peer_ip) = test_run.unwrap();
    let output_record = output_stream_queue[0].get_record();
    trace!("{:#?}", output_record);
    assert_eq!(output_stream_queue.len(), 1);

    // The outputted record is an ordered_record, meaning it is sorted
    // alphabetically on the key, so [0] is "peer_ip" and [1] is "prefix"
    assert_eq!(output_record.get_field_by_index(vec![1].into()).unwrap(), &prefix);
    assert_eq!(output_record.get_field_by_index(vec![0].into()).unwrap(), &peer_ip);
    trace!("{:#?}", output_stream_queue);
    assert_eq!(accept_reject, AcceptReject::Accept);
}