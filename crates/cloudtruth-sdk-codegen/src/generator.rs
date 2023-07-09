use std::{borrow::BorrowMut, cell::RefCell};

use crate::{
    api::{ApiOperation, ApiSpec},
    sdk::{
        methods::{SdkRootConstructor, SdkStaticRootConstructor},
        SdkObject,
    },
};

pub struct SdkGenerator {
    spec: ApiSpec,
    root_prefix: String,
}

impl SdkGenerator {
    pub fn new(spec: ApiSpec) -> Self {
        Self {
            spec,
            root_prefix: String::new(),
        }
    }

    pub fn spec(&self) -> &ApiSpec {
        &self.spec
    }

    pub fn root_prefix(&mut self, prefix: impl Into<String>) -> &mut Self {
        self.root_prefix = prefix.into();
        self
    }

    fn build_root_object<'a>(&'a self) -> SdkObject<'a> {
        let mut root = SdkObject::new("CloudtruthSdk");
        root.add_method(SdkRootConstructor::new(&root));
        root.add_method(SdkStaticRootConstructor::new());
        root
    }

    pub fn build_object_tree<'a>(&'a self) -> SdkObject<'a> {
        let operations = self.spec.operations().into_iter();
        let mut root = self.build_root_object();
        let mut ancestors: Vec<(&str, &mut SdkObject)> = Vec::with_capacity(operations.len());
        ancestors.push((self.root_prefix.as_ref(), &mut root));
        for op in operations {
            let uri = op.uri();
            let prefix = ancestors.last().unwrap().0;
            match uri.strip_prefix(prefix) {
                // same path, different method
                Some("") => {}
                // child path
                Some(path) => {
                    let parent = &mut *ancestors.last_mut().unwrap().1;
                    let current = SdkObject::new("foo");
                    parent.add_child(current);
                    ancestors.push((prefix, parent.children_mut().last_mut().unwrap()));
                    prefix = uri;
                }
                // unwind stack to find ancestor
                _ => {
                    while let Some((ancestor_prefix, ancestor)) = ancestors.pop() {
                        if uri.starts_with(ancestor_prefix) {
                            ancestors.push((ancestor_prefix, ancestor));
                            break;
                        }
                    }
                }
            }
        }
        root
    }
}

// pub fn longest_common_prefix<'a>(str1: &'a str, str2: &'a str) -> &'a str {
//     for (i, byte) in str1.as_bytes().iter().enumerate() {
//         if str2.as_bytes().get(i) != Some(byte) {
//             let mut end = i;
//             while !str1.is_char_boundary(end) {
//                 end -= 1;
//             }
//             return &str1[0..end];
//         }
//     }

//     // entire string is common prefix
//     str1
// }
