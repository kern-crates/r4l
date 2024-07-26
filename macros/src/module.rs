// SPDX-License-Identifier: GPL-2.0

use crate::helpers::*;
use proc_macro::{token_stream, Delimiter, TokenStream, TokenTree};

fn expect_string_array(it: &mut token_stream::IntoIter) -> Vec<String> {
    let group = expect_group(it);
    assert_eq!(group.delimiter(), Delimiter::Bracket);
    let mut values = Vec::new();
    let mut it = group.stream().into_iter();

    while let Some(val) = try_string(&mut it) {
        assert!(val.is_ascii(), "Expected ASCII string");
        values.push(val);
        match it.next() {
            Some(TokenTree::Punct(punct)) => assert_eq!(punct.as_char(), ','),
            None => break,
            _ => panic!("Expected ',' or end of array"),
        }
    }
    values
}

#[derive(Debug, Default)]
struct ModuleInfo {
    type_: String,
    license: String,
    name: String,
    author: Option<String>,
    description: Option<String>,
    initcall: String,
    alias: Option<Vec<String>>,
}

impl ModuleInfo {
    fn parse(it: &mut token_stream::IntoIter) -> Self {
        let mut info = ModuleInfo::default();
        info.initcall = ".initcall6.init".to_string();

        const EXPECTED_KEYS: &[&str] =
            &["type", "name", "author", "description", "license", "initcall", "alias"];
        const REQUIRED_KEYS: &[&str] = &["type", "name", "license"];
        let mut seen_keys = Vec::new();

        loop {
            let key = match it.next() {
                Some(TokenTree::Ident(ident)) => ident.to_string(),
                Some(_) => panic!("Expected Ident or end"),
                None => break,
            };

            if seen_keys.contains(&key) {
                panic!(
                    "Duplicated key \"{}\". Keys can only be specified once.",
                    key
                );
            }

            assert_eq!(expect_punct(it), ':');

            match key.as_str() {
                "type" => info.type_ = expect_ident(it),
                "name" => info.name = expect_string_ascii(it),
                "author" => info.author = Some(expect_string(it)),
                "description" => info.description = Some(expect_string(it)),
                "license" => info.license = expect_string_ascii(it),
                "initcall" => info.initcall = expect_string_initcall(it),
                "alias" => info.alias = Some(expect_string_array(it)),
                _ => panic!(
                    "Unknown key \"{}\". Valid keys are: {:?}.",
                    key, EXPECTED_KEYS
                ),
            }

            assert_eq!(expect_punct(it), ',');

            seen_keys.push(key);
        }

        expect_end(it);

        for key in REQUIRED_KEYS {
            if !seen_keys.iter().any(|e| e == key) {
                panic!("Missing required key \"{}\".", key);
            }
        }

        let mut ordered_keys: Vec<&str> = Vec::new();
        for key in EXPECTED_KEYS {
            if seen_keys.iter().any(|e| e == key) {
                ordered_keys.push(key);
            }
        }

        if seen_keys != ordered_keys {
            panic!(
                "Keys are not ordered as expected. Order them like: {:?}.",
                ordered_keys
            );
        }

        info
    }
}

pub(crate) fn module(ts: TokenStream) -> TokenStream {
    let mut it = ts.into_iter();
    let info = ModuleInfo::parse(&mut it);
    format!(
        "
            /// The module name.
            ///
            /// Used by the printing macros, e.g. [`info!`].
            const __LOG_PREFIX: &[u8] = b\"{name}\\0\";
            static THIS_MODULE: kernel::ThisModule = kernel::ThisModule();

            // Double nested modules, since then nobody can access the public items inside.
            mod __module_init {{
                mod __module_init {{
                    use super::super::{type_};
                    static mut __MOD: Option<{type_}> = None;
                    // Built-in modules are initialized through an initcall pointer
                    // and the identifiers need to be unique.
                    #[doc(hidden)]
                    #[link_section = \"{initcall_section}\"]
                    #[used]
                    pub static __{name}_initcall: extern \"C\" fn() -> core::ffi::c_int = __{name}_init;

                    #[doc(hidden)]
                    #[no_mangle]
                    pub extern \"C\" fn __{name}_init() -> core::ffi::c_int {{
                        // SAFETY: This function is inaccessible to the outside due to the double
                        // module wrapping it. It is called exactly once by the C side via its
                        // placement above in the initcall section.
                        unsafe {{ __init() }}
                    }}
                    
                    /// # Safety
                    ///
                    /// This function must only be called once.
                    unsafe fn __init() -> core::ffi::c_int {{
                        match <{type_} as kernel::Module>::init(&super::super::THIS_MODULE) {{
                            Ok(m) => {{
                                // SAFETY: No data race, since `__MOD` can only be accessed by this
                                // module and there only `__init` and `__exit` access it. These
                                // functions are only called once and `__exit` cannot be called
                                // before or during `__init`.
                                unsafe {{
                                    __MOD = Some(m);
                                }}
                                return 0;
                            }}
                            Err(e) => {{
                                return -1;
                            }}
                        }}
                    }}

                    /// # Safety
                    ///
                    /// This function must
                    /// - only be called once,
                    /// - be called after `__init` has been called and returned `0`.
                    unsafe fn __exit() {{
                        // SAFETY: No data race, since `__MOD` can only be accessed by this module
                        // and there only `__init` and `__exit` access it. These functions are only
                        // called once and `__init` was already called.
                        unsafe {{
                            // Invokes `drop()` on `__MOD`, which should be used for cleanup.
                            __MOD = None;
                        }}
                    }}
                }}
            }}
        ",
        type_ = info.type_,
        name = info.name,
        initcall_section = info.initcall
    )
    .parse()
    .expect("Error parsing formatted string into token stream.")
}
