use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

#[cfg(target_family = "wasm")]
pub fn random_range(min: usize, max: usize) -> usize {
    (random() * (max - min) as f64).floor() as usize + min
}

#[cfg(target_family = "wasm")]
pub fn random_range_descending(min: usize, max: usize) -> usize {
    (random() * random() * (max - min) as f64).floor() as usize + min
}

#[cfg(target_family = "wasm")]
pub fn random_velocity(min: f64, max: f64) -> f64 {
    (random() * (max - min)).floor() + min
}
#[cfg(target_family = "wasm")]
pub fn random_sign() -> f64 {
    if random() > 0.5 {
        return 1.0;
    }
    return -1.0;
}
