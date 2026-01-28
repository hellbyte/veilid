use super::*;

use jni::{objects::GlobalRef, objects::JObject, JNIEnv, JavaVM};
use lazy_static::*;

pub struct AndroidGlobals {
    pub vm: JavaVM,
    pub ctx: GlobalRef,
}

impl Drop for AndroidGlobals {
    fn drop(&mut self) {
        // Ensure we're attached before dropping GlobalRef
        self.vm.attach_current_thread_as_daemon().unwrap_or_log();
    }
}

lazy_static! {
    pub static ref ANDROID_GLOBALS: Arc<Mutex<Option<AndroidGlobals>>> = Arc::new(Mutex::new(None));
}

pub fn veilid_core_setup_android(env: JNIEnv, ctx: JObject) {
    *ANDROID_GLOBALS.lock() = Some(AndroidGlobals {
        vm: env.get_java_vm().unwrap_or_log(),
        ctx: env.new_global_ref(ctx).unwrap_or_log(),
    });
}

pub fn is_android_ready() -> bool {
    ANDROID_GLOBALS.lock().is_some()
}

pub fn get_android_globals() -> (JavaVM, GlobalRef) {
    let globals_locked = ANDROID_GLOBALS.lock();
    let globals = globals_locked.as_ref().unwrap_or_log();
    let env = globals.vm.attach_current_thread_as_daemon().unwrap_or_log();
    let vm = env.get_java_vm().unwrap_or_log();
    let ctx = globals.ctx.clone();
    (vm, ctx)
}
