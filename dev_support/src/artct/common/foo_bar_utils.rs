pub fn foo_core(a: String, b: i32, bar_res: String) -> String {
    let a = a + "-foo";
    let b = b + 3;
    format!("foo: a={}, b={}, bar=({})", a, b, bar_res)
}

pub fn bar_core(u: i32, v: String) -> String {
    let u = u + 1;
    let v = v + "-bar";
    format!("bar: u={}, v={}", u, v)
}
