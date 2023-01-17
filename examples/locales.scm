(call_expression
  function: [
    (identifier) @ident
    (member_expression
      property: (_) @ident
    )
  ] @func
  arguments: (arguments
    .
    (string (string_fragment) @key)
  )
  (#match? @ident "^tx?$"))
