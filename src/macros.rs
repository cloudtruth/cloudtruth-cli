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
