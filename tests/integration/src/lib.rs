#![allow(clippy::should_panic_without_expect, dead_code)] // 1.73.0
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
pub mod empty_doc_mod {
    ///```
    ///```
    mod private_mod {}

    ///```
    ///```
    pub struct Item;
}

pub struct Struct;

/// ```
/// let _ = integration::Struct;
/// ```
pub mod normal_doc_mod {
    ///```
    /// let _ = integration::Struct;
    ///```
    mod private_mod {
        ///```
        /// let _ = integration::Struct;
        ///```
        struct Item {}
    }

    ///```
    /// let _ = integration::Struct;
    ///```
    pub struct Item {}
}

mod attribute {
    /// ```ignore
    /// ```
    fn ignore() {}

    /// ```should_panic
    /// assert!(false);
    /// ```
    fn should_panic() {}

    /// `no_run` attribute will compile your code but not run it
    ///```no_run
    ///```
    fn no_run() {}

    /// ```compile_fail
    /// let x = 5;
    /// x += 2; // shouldn't compile!
    /// ```
    fn should_compile_fail() {}

    /// ```compile_fail
    /// ```
    fn should_compile_fail_but_didnt() {}

    /// ```edition2018
    /// ```
    fn edition2018() {}
}
