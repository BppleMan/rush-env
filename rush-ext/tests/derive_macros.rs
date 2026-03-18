use rush_ext::{FieldName, Getter, MutGetter, Setter};

#[derive(Debug, FieldName)]
#[allow(dead_code)]
struct FieldNameProbe {
    count: usize,
    label: String,
    r#type: &'static str,
}

#[derive(Debug, FieldName, Getter, MutGetter, Setter)]
struct AccessorProbe {
    count: usize,
    label: String,
    enabled: bool,
}

#[test]
fn field_name_macro_returns_struct_field_names() {
    assert_eq!(FieldNameProbe::field_count(), "count");
    assert_eq!(FieldNameProbe::field_label(), "label");
    assert_eq!(FieldNameProbe::field_type(), "type");
}

#[test]
fn getter_setter_and_mut_getter_work_together() {
    let mut probe = AccessorProbe {
        count: 1,
        label: "alpha".to_string(),
        enabled: true,
    };

    assert_eq!(AccessorProbe::field_count(), "count");
    assert_eq!(AccessorProbe::field_label(), "label");
    assert_eq!(AccessorProbe::field_enabled(), "enabled");
    assert_eq!(*probe.get_count(), 1);
    assert_eq!(probe.get_label(), "alpha");
    assert!(*probe.get_enabled());

    probe.set_count(2).set_label("beta".to_string()).set_enabled(false);
    probe.label_mut().push('!');

    assert_eq!(*probe.get_count(), 2);
    assert_eq!(probe.get_label(), "beta!");
    assert!(!*probe.get_enabled());
}
