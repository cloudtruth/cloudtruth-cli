use std::rc::Rc;

use crate::{
    api::ApiSpec,
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

    pub fn build_object_tree(&self) -> Rc<SdkObject> {
        let mut root = Rc::new(SdkObject::new("CloudtruthSdk"));
        let operations = self.spec.operations().iter();
        let mut ancestors: Vec<(&str, Rc<SdkObject>)> = Vec::with_capacity(operations.len());
        ancestors.push((self.root_prefix.as_ref(), root.clone()));
        for op in operations {
            let uri = op.uri();
            let prefix = ancestors.last().unwrap().0;
            match uri.strip_prefix(prefix) {
                // same path, different method
                Some("") => {}
                // child path
                Some(_path) => {
                    let mut parent = ancestors.last().unwrap().1.clone();
                    let current = Rc::new(SdkObject::new("foo"));
                    Rc::make_mut(&mut parent).add_child(current);
                    ancestors.push((prefix, parent.children().last().unwrap().clone()));
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
        let root_ref = Rc::make_mut(&mut root);
        root_ref.add_method(SdkRootConstructor::new(root_ref.name().clone()));
        root_ref.add_method(SdkStaticRootConstructor::new());
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
