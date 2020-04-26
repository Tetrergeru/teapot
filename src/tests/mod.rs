
#[test]
fn it_works() {
    super::teapot::listen("", Vec::<(String, &dyn super::teapot::RequestHandler)>::new());
}