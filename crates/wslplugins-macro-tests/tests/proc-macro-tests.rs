#[cfg(test)]
mod test {
    use trybuild::TestCases;
    #[test]
    fn test_macro_sucess() {
        let t = TestCases::new();
        t.pass("tests/ui/success.rs")
    }
}
