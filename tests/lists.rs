use log::trace;
use roto::ast::AcceptReject;
use roto::compiler::Compiler;

use roto::blocks::Scope::{self, FilterMap};
use roto::types::collections::Record;
use roto::types::typedef::TypeDef;
use roto::types::typevalue::TypeValue;
use roto::vm::{self, VmResult};

use rotonda_store::prelude::MergeUpdate;

use routecore::asn::Asn;

mod common;

#[derive(Debug, Clone)]
struct RibValue(Vec<TypeValue>);

impl MergeUpdate for RibValue {
    type UserDataIn = ();
    type UserDataOut = ();

    fn merge_update(
        &mut self,
        update_record: RibValue,
        _: Option<&Self::UserDataIn>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.0 = update_record.0;
        Ok(())
    }

    fn clone_merge_update(
        &self,
        update_meta: &Self,
        _: Option<&Self::UserDataIn>,
    ) -> Result<(Self, Self::UserDataOut), Box<dyn std::error::Error>>
    where
        Self: std::marker::Sized,
    {
        let mut new_meta = update_meta.0.clone();
        new_meta.push(self.0[0].clone());
        Ok((RibValue(new_meta), ()))
    }
}

impl std::fmt::Display for RibValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

fn src_code(code_line: &str, end_accept_reject: &str) -> String {
    let pre = format!(
        r###"
        filter-map in-filter-map {{
            define {{
                rx_tx msg: BmpMsg;
                a = 100;
                b = [10,20,30,40];
                c = 99;
            }}

            term peer-asn-matches {{
                match {{
                    {}
                }}
            }}

            apply {{
                filter match peer-asn-matches matching {{ return {}; }};
                return accept;
            }}
        }}

        type BmpMsg {{
            type: U8,
            asn: Asn
        }}
    "###,
        code_line, end_accept_reject
    );

    pre
}

fn test_data(
    name: Scope,
    source_code: &str,
) -> Result<VmResult, Box<dyn std::error::Error>> {
    trace!("Evaluate filter-map {}...", name);

    let c = Compiler::new();
    let compiler_res = c.build_from_compiler(source_code);

    if let Err(e) = &compiler_res {
        eprintln!("{e}");
    }

    let roto_packs = compiler_res?;
    let roto_pack = roto_packs.retrieve_pack_as_refs(&name)?;
    let asn: TypeValue = Asn::from_u32(211321).into();

    println!("ASN {:?}", asn);

    let my_rec_type = TypeDef::new_record_type(vec![
        ("type", Box::new(TypeDef::U8)),
        ("asn", Box::new(TypeDef::Asn)),
    ])
    .unwrap();

    let my_payload = Record::create_instance_with_ordered_fields(
        &my_rec_type,
        vec![("type", TypeValue::from(1_u8)), ("asn", asn)],
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

    let mut vm = vm::VmBuilder::new()
        .with_data_sources(roto_pack.data_sources)
        .with_mir_code(roto_pack.mir)
        .build()?;

    let res = vm.exec(my_payload, None::<Record>, None, mem).unwrap();

    println!("\nRESULT");
    println!("action: {}", res.accept_reject);
    println!("rx    : {:?}", res.rx);
    println!("tx    : {:?}", res.tx);

    Ok(res)
}

#[test]
fn test_list_compare_1() {
    common::init();
    let src_line = src_code("msg.type in [2,3,4,5]; // Peer Down", "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);

    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_list_compare_2() {
    common::init();
    let src_line =
        src_code("msg.type in [1,2,3,4,5]; // Peer Down", "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);

    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Reject);
}

#[test]
fn test_list_compare_3() {
    common::init();
    let src_line =
        src_code("msg.type in [1,2,3,4,5]; // Peer Down", "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);

    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Reject);
}

#[test]
fn test_list_compare_4() {
    common::init();
    let src_line =
        src_code("msg.type in [2,3,4,5,1,9]; // Peer Down", "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);

    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Reject);
}

#[test]
fn test_list_compare_5() {
    common::init();
    let src_line =
        src_code(r#""stringetje" in [2,3,4,5,1]; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);

    test_run.unwrap_err();
}

#[test]
fn test_list_compare_6() {
    common::init();
    let src_line =
        src_code(r#"msg.type not in [2,3,4,5,1]; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);
    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Reject);
}

#[test]
fn test_list_compare_7() {
    common::init();
    let src_line = src_code(r#"a in [2,3,4,5,1]; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);
    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_list_compare_8() {
    common::init();
    let src_line = src_code(r#"100 in [2,3,4,5,1]; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);
    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_list_compare_9() {
    common::init();
    let src_line = src_code(r#"100 in b; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);
    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Accept);
}

#[test]
fn test_list_compare_10() {
    common::init();
    let src_line = src_code(r#"100 in [2,3,4,a]; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);
    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Reject);
}

#[test]
fn test_list_compare_11() {
    common::init();
    let src_line = src_code(r#"100 in [2,3,4,c]; // Peer Down"#, "reject");
    let test_run = test_data(FilterMap("in-filter-map".into()), &src_line);
    let VmResult { accept_reject, .. } = test_run.unwrap();
    assert_eq!(accept_reject, AcceptReject::Accept);
}
