use crate::{ context::Context, prelude::* };

pub fn info(context: &Context, msg: &str) {
  context.window.alert_with_message(msg).unwrap();
}

pub fn error(context: &Context, msg: &str) -> ! {
  context.window.alert_with_message(msg).unwrap();
  context.window.alert_with_message("由于先前的错误，程序已经无法继续正常运行，若希望重试请先刷新页面。").unwrap();
  panic!();
}
