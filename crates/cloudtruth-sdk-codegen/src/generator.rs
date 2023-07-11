use std::rc::Rc;

use crate::{
    api::ApiSpec,
    sdk::{
        methods::{SdkApiMethod, SdkRootConstructor, SdkStaticRootConstructor},
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

    pub fn build_objects(&self) -> Vec<Rc<SdkObject>> {
        // list of objects that we are building
        let mut objects = Vec::new();
        // iterator over API operations from the spec (assumed to be sorted)
        let operations = self.spec.operations();
        // a stack of ancestors from pervious iterations
        let mut ancestors: Vec<(&str, Rc<SdkObject>)> = Vec::with_capacity(operations.len());
        // create root SDK object
        let mut root = SdkObject::new("CloudtruthSdk");
        root.add_method(SdkRootConstructor::new(root.name().clone()));
        root.add_method(SdkStaticRootConstructor::new());
        // add root to ancestor stack
        ancestors.push((self.root_prefix.as_ref(), Rc::new(root)));

        for op in operations.iter() {
            // println!();
            let uri = op.uri().trim_end_matches('/');
            // let method = op.http_method();
            // println!("{method} {uri}");

            // find the ancestor of current path in the stack and get the descendant path segments
            let descendant_path = loop {
                match ancestors.last() {
                    Some((ancestor_prefix, _)) => match uri.strip_prefix(ancestor_prefix) {
                        // found ancestor, return the descendant path
                        Some(descendant_path) => break descendant_path,
                        // not an ancestor, pop from stack and append to our output list
                        None => objects.push(ancestors.pop().unwrap().1),
                    },
                    // no valid anceestor (unexpected behavior)
                    None => panic!("No ancestor found for {uri}"),
                }
            };
            for child_segment in descendant_path.split('/').skip(1) {
                if child_segment.is_empty()
                    || child_segment.starts_with('{') && child_segment.ends_with('}')
                {
                    let size = ancestors.len();
                    if let Some((_, previous_object)) = ancestors.get_mut(size - 1) {
                        Rc::get_mut(previous_object)
                            .expect("Could not add method to object because other references to this object were found")
                            .add_method(SdkApiMethod::new(op.clone()));
                    }
                } else {
                    let current_object = Rc::new(SdkObject::new(child_segment));
                    // append this path segment to current prefix
                    let segment_start = child_segment.as_ptr() as usize - uri.as_ptr() as usize;
                    let segment_end = segment_start + child_segment.len();
                    let path = &uri[..segment_end];
                    ancestors.push((path, current_object));
                }
            }
        }
        // add any remaining ancestors in stack to output list
        objects.extend(ancestors.into_iter().map(|(_, ancestor)| ancestor));
        objects
    }
}
