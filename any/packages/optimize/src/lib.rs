use proc_macro::TokenStream;
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use proc_macro2::Span;
use quote::quote;
use syn::{ItemFn, LitStr, Signature, parse_macro_input};
use temp_dir::TempDir;

/// Optimizes the function in a separate crate for size and then replaces it with
/// the optimized assembly
///
/// # Limitations
/// - the function must use the `extern "C"` ABI
/// - the function cannot use any API outside of the `core` crate
/// - the function cannot rely on import *outside* of it; imports must be placed inside the function
#[proc_macro_attribute]
pub fn size(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(attr.is_empty(), "`#[optimized]` takes no parameters");
    let item_fn = parse_macro_input!(item as ItemFn);
    validate_item(&item_fn);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set by Cargo"));
    let target = extract_target(&out_dir);

    let assembly = produce_assembly(&item_fn, &target);

    let assembly = LitStr::new(&assembly, Span::call_site());
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = &item_fn;
    quote!(
        // emit the source code so we can lint it
        const _: () = {
            #vis #sig {
                #block
            }
        };

        #(#attrs)*
        #[unsafe(naked)]
        #vis #sig {
            core::arch::naked_asm!(#assembly)
        }
    )
    .into()
}

fn extract_target(path: &Path) -> String {
    let mut components = path.components().rev();
    for component in components.by_ref() {
        if component == Component::Normal("build".as_ref()) {
            break;
        }
    }
    components.next(); // profile
    if let Some(Component::Normal(target)) = components.next() {
        return target.to_string_lossy().into_owned();
    }

    panic!("could not determine compilation target")
}

fn validate_item(item: &ItemFn) {
    let ItemFn {
        attrs: _,
        vis: _,
        sig,
        block: _,
    } = item;
    let Signature {
        constness,
        asyncness,
        unsafety: _,
        abi,
        fn_token: _,
        ident: _,
        generics,
        paren_token: _,
        inputs: _,
        variadic,
        output: _,
    } = sig;
    assert!(constness.is_none(), "function must not be `const`");
    assert!(asyncness.is_none(), "function must not be `async`");
    assert!(
        generics.lt_token.is_none()
            && generics.params.is_empty()
            && generics.gt_token.is_none()
            && generics.where_clause.is_none(),
        "function must have no generics"
    );
    let abi = abi
        .as_ref()
        .and_then(|abi| abi.name.as_ref())
        .expect(r#"ABI must be `extern "C"`"#);
    assert_eq!("C", abi.value());
    assert!(variadic.is_none(), "function must not be variadic");
}

fn produce_assembly(item_fn: &ItemFn, target: &str) -> String {
    let temp_dir = TempDir::new().expect("could not create temporary directory");
    let base_dir = temp_dir.path();
    let input_rs = base_dir.join("input.rs");

    let krate = quote!(
        #![no_std]
        #![no_main]

        #[unsafe(export_name = "_start")]
        #item_fn

        #[panic_handler]
        fn never(_: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    );
    fs::write(&input_rs, krate.to_string()).expect("could not create `input.rs`");

    let rustc = env::var("RUSTC");
    let rustc = rustc.as_deref().unwrap_or("rustc");
    let status = Command::new(rustc)
        .args([
            "--target",
            target,
            "--crate-type=bin",
            "--emit=asm",
            "-Copt-level=z",
            "-Clto=fat",
            "-Ccodegen-units=1",
            "input.rs",
        ])
        .current_dir(base_dir)
        .status()
        .expect("`rustc` not found");
    assert!(status.success(), "`rustc` invocation failed");

    let assembly = fs::read_to_string(input_rs.with_extension("s"))
        .expect("could not read output assembly file");
    let labels_dict = compute_labels_dictionary(&assembly);
    let mut saw_fnstart = false;
    let mut saw_fnend = false;
    let mut output = String::new();
    for line in assembly.lines() {
        if line.trim() == ".fnstart" {
            assert!(!saw_fnstart, "more than one function in output assembly");
            saw_fnstart = true;
            output.push_str(line);
            output.push('\n');
        } else if line.trim() == ".fnend" {
            assert!(!saw_fnend, "more than one function in output assembly");
            saw_fnend = true;
            output.push_str(line);
            output.push('\n');
        } else if line.trim().starts_with(".size") {
            continue;
        } else if saw_fnstart && !saw_fnend {
            output.push_str(&sanitize_assembly_line(line, &labels_dict));
            output.push('\n');
        }
    }

    output
}

type LabelDict<'a> = BTreeMap<&'a str, String>;

fn compute_labels_dictionary(assembly: &str) -> LabelDict<'_> {
    let mut hasher = DefaultHasher::new();
    assembly.hash(&mut hasher);
    let hash = hasher.finish();
    let mut dict = BTreeMap::new();
    for line in assembly.lines() {
        if let Some((label, _)) = line.split_once(':')
            && label.starts_with(".L")
        {
            dict.insert(label, format!("{label}{hash}"));
        }
    }

    dict
}

fn sanitize_assembly_line(line: &str, dict: &LabelDict) -> String {
    let mut output = line.replace('{', "{{").replace('}', "}}");
    for (k, v) in dict {
        output = output.replace(k, v);
    }
    output
}
