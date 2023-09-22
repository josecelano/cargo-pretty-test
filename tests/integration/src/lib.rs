#[test]
fn works() {}

mod submod {
    mod panic {
        #[test]
        fn panicked() {
            panic!()
        }

        #[test]
        #[should_panic = "explicit panic"]
        fn should_panic() {
            panic!()
        }

        #[test]
        #[should_panic]
        #[allow(clippy::should_panic_without_expect)]
        fn should_panic_without_reanson() {
            panic!()
        }
    }

    #[test]
    fn normal_test() {}

    #[test]
    #[ignore = "reason"]
    fn ignore() {}

    #[test]
    #[ignore]
    fn ignore_without_reason() {}
}
