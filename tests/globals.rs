mod modules;


//------------ RIB parse ----------------------------------------------------

#[test]
fn test_rib_invalid_1() {
    let compiler = modules::TestCompiler::create(
        "test_rib_invalid_1",
        r###"
            rib my_rib contains Blaffer { 
                bla: Bla, blow_up 
            }
            // comment
            "###,
    );

    compiler.test_parse(false);
}

#[test]
fn test_rib_invalid_2() {
    let compiler = modules::TestCompiler::create(
        "invalid-rib-with-comment-2",
        r###"
            rib my_rib contains Blaffer { 
                bla: Bla; blow: up
            }
            // comment
            "###
    );

    compiler.test_parse(false);
}

#[test]
fn test_rib_without_name_1() {
    let compiler = modules::TestCompiler::create(
        "rib-without-a-name",
        r###"
        // comment
        rib {}
    "###);
    compiler.test_parse(false);
}

#[test]
fn test_empty() {
    let compiler = modules::TestCompiler::create(
        "invalid-rib-without-name-2",
        r###"
            // some comment
            // bl alba  bcomment"
        "###
    );

    compiler.test_parse(false);
}

#[test]
fn test_interspersed_comments() {
    let compiler = modules::TestCompiler::create(
        "interspersed-comments",
        r###"
        rib my_rib contains SomeCrap { prefix: Prefix, as-path: AsPath }
        // comment
        rib unrib contains Blaffer { _ip: IpAddress }
        "###,
    );

    let _p = compiler.test_parse(true);
    let _e = _p.test_eval(true);
}