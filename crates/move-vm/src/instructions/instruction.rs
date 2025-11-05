use crate::instructions::function_call::FunctionCall;
use crate::instructions::publish_modules::PublishModules;

pub enum Instruction {
    MethodCall(FunctionCall),
    PublishModules(PublishModules),
}
