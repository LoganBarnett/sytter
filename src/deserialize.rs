use toml::Value;

pub fn i32_des(x: &Value) -> Option<i32> {
    x.as_integer().map(|xx| -> i32 { xx as i32 })
}

pub fn vec_i32_des(o: Option<&Value>) -> Vec<i32> {
    o.and_then(|x| x.as_array())
        .and_then(|ys| -> Option<Vec<i32>> {
            ys.iter().map(i32_des).collect::<Option<Vec<i32>>>()
        })
        .unwrap_or(vec![0_i32])
}
