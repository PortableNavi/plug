use keep::*;


#[test]
fn keep_roundtrip()
{
    let keep_a = Keep::new(39);
    assert_eq!(39, *keep_a.read());
}


#[test]
fn read_twice()
{
    let keep = Keep::new(39);
    assert_eq!(39, *keep.read());
    assert_eq!(39, *keep.read());
}


#[test]
fn keep_swapping()
{
    let keep_a = Keep::new(39);
    let keep_b = Keep::new(2);

    assert_eq!(39, *keep_a.read());
    assert_eq!(2, *keep_b.read());

    keep_a.swap_with(&keep_b);

    assert_eq!(2, *keep_a.read());
    assert_eq!(39, *keep_b.read());

    drop(keep_a);

    assert_eq!(39, *keep_b.read());
}


#[test]
fn keep_clone()
{
    let original = Keep::new(39);
    let cloned = original.clone();

    assert_eq!(39, *cloned.read());
    drop(original);
    assert_eq!(39, *cloned.read());
}


#[test]
fn guards_keep_value_alive()
{
    let keep = Keep::new(39);
    let guard = keep.read();

    drop(keep);
    assert_eq!(39, *guard);
}


#[test]
fn correct_drop_behavior()
{
    // Drop guard first
    {
        let keep = Keep::new(39);
        let guard = keep.read();
        drop(guard);
        assert_eq!(39, *keep.read());
    }

    // Drop keep first
    {
        let keep = Keep::new(39);
        let guard = keep.read();
        drop(keep);
        assert_eq!(39, *guard);
    }
}


#[test]
fn multiple_guards_outlive_keep()
{
    let keep = Keep::new(39);

    let guard_a = keep.read();
    let guard_b = keep.read();
    let guard_c = keep.read();

    drop(keep);

    let guard_d = guard_b.clone();

    assert_eq!(39, *guard_a);
    assert_eq!(39, *guard_b);
    assert_eq!(39, *guard_c);
    assert_eq!(39, *guard_d);
}


#[test]
fn multiple_guards()
{
    let keep = Keep::new(39);

    // Making sure, that the guards drop before the keep using this scope
    {
        let guard_a = keep.read();
        let guard_b = keep.read();
        let guard_c = keep.read();
        let guard_d = guard_b.clone();

        assert_eq!(39, *guard_a);
        assert_eq!(39, *guard_b);
        assert_eq!(39, *guard_c);
        assert_eq!(39, *guard_d);
    }
}


#[test]
fn write()
{
    let keep = Keep::new(39);

    let old = keep.read();
    keep.write(14);
    let new = keep.read();

    assert_eq!(39, *old);
    assert_eq!(14, *new);
}


#[test]
fn swap()
{
    let keep = Keep::new(39);

    let old = keep.swap(14);
    let new = keep.read();

    assert_eq!(39, *old);
    assert_eq!(14, *new);
}


#[test]
fn exchange()
{
    let keep_ok = Keep::new(39);
    let keep_err = Keep::new("mk");

    let guard_ok = keep_ok.read();
    let guard_err = keep_err.swap("???");

    let ok = keep_ok.exchange(&guard_ok, 10).unwrap();
    let err = keep_err.exchange(&guard_err, "oh no...").unwrap_err();

    assert_eq!(10, *keep_ok.read());
    assert_eq!("???", *keep_err.read());

    assert_eq!(39, *ok);
    assert_eq!("???", *err);
}
