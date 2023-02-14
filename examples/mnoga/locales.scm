; Match direct calls to t, either as a bare identifier or as a
; property lookup.
;
;     t("a")
;     foo.t("b");
;     foo.bar.t("c")
(call_expression
  function: [
    (identifier) @ident
    (member_expression
      property: (_) @ident
    )
  ]
  arguments: (arguments
    .
    (string (string_fragment) @key)
  )
  (#match? @ident "^tx?$"))

; Match calls to t.bind or t.call, where t itself, like before, is a
; bare identifier or property lookup
;
;    t.call(this, "a")
;    foo.t.call(this, "b");
;    foo.bar.t.call(this, "c");
;    a.b.c.d.e.f.g.h.i.j.k.t.call("this", "d");
;
;    t.bind(this, "a")
;    foo.t.bind(this, "b");
;    foo.bar.t.bind(this, "c");
;    a.b.c.d.e.f.g.h.i.j.k.t.bind(a, "d")
(call_expression
  function: [
    (member_expression
      object: [
        (identifier) @ident
        (member_expression
          property: (_) @ident
        )
      ]
      property: (_) @property
    )
  ]
  arguments: (arguments
    .
    (_)
    (string (string_fragment) @key)
  )
  (#match? @ident "^tx?$")
  (#match? @property "^(bind|call)$"))
