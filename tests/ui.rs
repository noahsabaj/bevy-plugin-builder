use trybuild::TestCases;

#[test]
fn compile_fail_tests() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/compile_fail/*.rs");
}
