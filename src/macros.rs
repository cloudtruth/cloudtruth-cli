#![allow(unused_macros)]

// Intended to be used in partial handling of errors. Assuming an enum is used as the error object,
// a local handler can match against the enum and handle the variants that require special error-
// handling. This macro can be used in the catch-all branch to propagate the error out of the caller
// for the global error handler to deal with. We use a macro for this operation in order to avoid
// problems where wrapping the error object can result in the location information adding extra
// stack traces and so we don't need to litter the code with calls into the exception handling
// library.
macro_rules! propagate_error {
    ($err:ident) => {{
        return Err(From::from($err));
    }};
}

// The graphql_client library unfortunately does not generate common structs for user-defined types
// emanating from the GraphQL schema. In particular, every mutation in CloudTruth offers two levels
// of error reporting. There is a top-level `errors` field, which is used for critical errors in
// executing the query that cannot be rectified by a user. Then there is an inner `errors` field
// that is part of the definition for the resource being returned by the mutation. This inner-level
// of error reporting is used for things like validation error messages corresponding to actions
// that an end-user can rectify.
//
// Since every mutation response includes this additional level of error-handling, it would be very
// convenient if we could have common error handling code. Unfortunately, due to the way that the
// graphql_client generates code, each mutation response will create its own copy of the `UserError`
// type from the schema IDL. The purpose of this macro is to take the values from the distinct error
// structs and convert them into a common struct object that can be used by a centralized error
// handler.
//
// See https://github.com/graphql-rust/graphql-client/issues/356 for more details.
macro_rules! to_user_errors {
    ($errors:expr) => {{
        $errors
            .into_iter()
            .map(|error| crate::graphql::UserError {
                message: error.message,
                path: error.path,
            })
            .collect::<Vec<crate::graphql::UserError>>()
    }};
}
