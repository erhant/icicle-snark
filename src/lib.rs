mod cache;
mod conversions;
mod file_wrapper;
mod icicle_helper;
mod proof_helper;
mod zkey;

pub use cache::{CacheManager, ZKeyCache};
use file_wrapper::FileWrapper;
use icicle_bn254::curve::{CurveCfg, G2CurveCfg, ScalarField};
use icicle_core::curve::{Affine, Projective};
use icicle_runtime::eIcicleError;
use proof_helper::groth16_prove_helper;
use std::time::Instant;
use std::path::Path;

pub type F = ScalarField;
pub type C1 = CurveCfg;
pub type C2 = G2CurveCfg;
pub type G1 = Affine<C1>;
pub type G2 = Affine<C2>;
pub type ProjectiveG1 = Projective<C1>;
pub type ProjectiveG2 = Projective<C2>;

/// Attempts to load and set a backend device.
///
/// Possible device names:
/// - `CPU`
/// - `CUDA`
fn try_load_and_set_backend_device(device_type: &str) -> Result<(), eIcicleError> {
    if device_type != "CPU" {
        icicle_runtime::runtime::load_backend_from_env_or_default()?;
    }
    let device = icicle_runtime::Device::new(device_type, 0 /* =device_id*/);
    icicle_runtime::set_device(&device)
}

pub fn groth16_prove(
    witness_path: impl AsRef<Path>,
    zkey_path: impl AsRef<Path>,
    proof_path: impl AsRef<Path>,
    public_path: impl AsRef<Path>,
    device: &str,
    cache_manager: &mut CacheManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    try_load_and_set_backend_device(device);

    let cache_key = format!("{:?}_{}", zkey_path.as_ref(), device);

    // load from cache w.r.t zkey and device
    let cache_key = format!("{}_{}", zkey_path.as_ref().display(), device);
    let cache_key = format!("{}_{}", zkey_path.as_ref().display(), device);
    if !cache_manager.contains(&cache_key) {
        let computed_cache = cache_manager.compute(zkey_path)?;
        cache_manager.insert_cache(&cache_key, computed_cache);
    }
    let zkey_cache = cache_manager.get_cache(&cache_key);

    let (proof_data, public_signals) = groth16_prove_helper(witness_path, zkey_cache)?;

    FileWrapper::save_json_file(proof_path, &proof_data)?;
    FileWrapper::save_json_file(public_path, &public_signals)?;

    Ok(())
}
