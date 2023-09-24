#![allow(clippy::should_panic_without_expect)] // 1.73.0
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
        fn should_panic_but_didnt() {}

        #[test]
        #[should_panic]
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

/// ```
/// ```
pub mod doc {}
