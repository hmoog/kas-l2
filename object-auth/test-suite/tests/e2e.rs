use kas_l2_object_auth_capabilities::{AuthContext, AuthResult, ObjectLock};
use kas_l2_object_auth_signature_lock::SignatureKey;
use kas_l2_object_auth_threshold_lock::{ThresholdKey, ThresholdLock};

#[test]
pub fn test_object_auth() -> AuthResult<()> {
    // init some locks
    let lock1 = ObjectLock::new::<SignatureKey, _>(4);
    let lock2 = ObjectLock::new::<SignatureKey, _>(2);

    let threshold_lock = ObjectLock::new::<ThresholdKey, i32>(ThresholdLock::new(
        vec![
            ObjectLock::new::<SignatureKey, _>(4),
            ObjectLock::new::<SignatureKey, _>(7),
            ObjectLock::new::<SignatureKey, _>(9),
        ],
        2,
    ));

    // set up runtime from mocked tx context
    let mut auth_ctx = AuthContext::new(&[1, 2, 3]);
    auth_ctx.register_key_type::<SignatureKey>();
    auth_ctx.register_key_type::<ThresholdKey>();

    auth_ctx.retrieve_key(&lock1).expect_err("should fail");
    auth_ctx.retrieve_key(&lock2)?.try_into::<SignatureKey>().expect("should work");

    let lock3 = auth_ctx.new_lock::<SignatureKey>(7);
    auth_ctx.retrieve_key(&lock3).expect_err("should fail");

    auth_ctx.retrieve_key(&threshold_lock).expect_err("should fail");

    let mut auth_ctx = AuthContext::new(&[1, 2, 3, 4, 7]);
    auth_ctx.register_key_type::<SignatureKey>();
    auth_ctx.register_key_type::<ThresholdKey>();

    auth_ctx.retrieve_key(&threshold_lock)?.try_into::<ThresholdKey>().expect("should work");

    Ok(())
}
