use std::rc::Rc;

use syn::parse_quote;

use crate::{
    api::ApiSpec,
    sdk::{
        methods::{
            SdkApiMethod, SdkChildConstructor, SdkRootConstructor, SdkStaticRootConstructor,
        },
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
        // iterator over API operations from the spec (assumed to be sorted)
        let operations = self.spec.operations();
        // list of objects that we are building
        let mut objects = Vec::with_capacity(operations.len());
        // a stack of ancestors from pervious iterations
        let mut ancestors = Vec::with_capacity(operations.len());
        // create root SDK object
        let mut root = SdkObject::new("CloudtruthSdk", None);
        root.add_field("client", parse_quote![Arc<Client>]);
        root.add_method(SdkRootConstructor::new(&root));
        root.add_method(SdkStaticRootConstructor::new());
        // add root to ancestor stack
        ancestors.push((self.root_prefix.as_ref(), root));

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
                        None => objects.push(Rc::new(ancestors.pop().unwrap().1)),
                    },
                    // no valid ancestor (unexpected behavior)
                    None => panic!("No ancestor found for {uri}"),
                }
            };
            // println!("{descendant_path:#}");
            for child_segment in descendant_path.trim_start_matches('/').split('/') {
                if child_segment.is_empty() {
                    continue;
                }
                let is_path_var = child_segment.starts_with('{') && child_segment.ends_with('}');
                let name = if is_path_var {
                    child_segment
                        .chars()
                        .filter(|c| *c == '_' || c.is_alphanumeric())
                        .collect::<String>()
                } else {
                    child_segment.to_string()
                };
                // append this path segment to current prefix
                let segment_start = child_segment.as_ptr() as usize - uri.as_ptr() as usize;
                let segment_end = segment_start + child_segment.len();
                let path = &uri[..segment_end];
                // get parent object
                let size = ancestors.len();
                let parent_object = ancestors.get_mut(size - 1).map(|(_, obj)| obj);
                // create SDK obect for this node
                let mut current_object = SdkObject::new(
                    path.strip_prefix(&self.root_prefix).unwrap(),
                    parent_object.as_deref(),
                );
                // attach struct field for the path variable
                if is_path_var {
                    current_object.add_field(&name, parse_quote![Arc<str>]);
                }

                // attach getter method to parent object
                if let Some(parent_object) = parent_object {
                    let mut method = SdkChildConstructor::new(parent_object, &current_object);
                    if is_path_var {
                        method.add_arg(&name, parse_quote![impl Into<Arc<str>>]);
                    }
                    parent_object.add_method(method);
                }

                // add to ancestors stack
                ancestors.push((path, current_object));
            }
            let size = ancestors.len();
            if let Some((_, last_object)) = ancestors.get_mut(size - 1) {
                last_object.add_method(SdkApiMethod::new(uri, op.clone()));
            }
        }

        // add any remaining ancestors in stack to output list
        objects.extend(ancestors.into_iter().map(|(_, ancestor)| Rc::new(ancestor)));
        objects
    }
}
