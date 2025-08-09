use rush_var::*;

fn main() {
    let mut env = Env::new();
    let opts = Options::default();

    env.set_scalar("JUST_NAME", "filename");
    let result = expand_str("${JUST_NAME:h}", &env, &opts).unwrap();
    println!("JUST_NAME:h = '{}'", result);

    env.set_scalar("ROOT_PATH", "/");
    let result = expand_str("${ROOT_PATH:h}", &env, &opts).unwrap();
    println!("ROOT_PATH:h = '{}'", result);
}
