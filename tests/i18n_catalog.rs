//! ot3's string catalog through `domain`: `i18n.t`, `i18n.catalog`,
//! `i18n.resolve` and `i18n.languages` over `data/i18n.json`.
use serde_json::json;
use t3_protocol::domain::call;

#[test]
fn the_catalog_is_ot3s_twenty_languages() {
    let c = t3_protocol::i18n::catalog();
    let locales = c.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    let mut expected = t3_protocol::i18n::LANGUAGES.iter().map(|(id, _)| id.to_string()).collect::<Vec<_>>();
    expected.sort();
    assert_eq!(locales, expected);
    let sources = c["en"].as_object().unwrap();
    assert_eq!(sources.len(), 2027, "Desktop catalog plus retained compatible sources");
    for (locale, strings) in c.as_object().unwrap() {
        let strings = strings.as_object().unwrap();
        assert_eq!(strings.len(), sources.len(), "{locale}");
        assert!(sources.keys().all(|k| strings[k].as_str().is_some_and(|s| !s.is_empty())), "{locale}");
    }
    // English maps every source to itself.
    assert!(sources.iter().all(|(k, v)| v == k));
}

#[test]
fn t_catalog_resolve_and_languages() {
    assert_eq!(call("i18n.t", &json!({"locale":"es_CL.UTF-8","key":"Close"})).unwrap(), "Cerrar");
    assert_eq!(call("i18n.t", &json!({"locale":"de","key":"{n} subagents","args":{"n":3}})).unwrap(), "3 Unteragenten");
    assert_eq!(call("i18n.t", &json!({"locale":"zz","key":"Close"})).unwrap(), "Close");
    assert_eq!(call("i18n.t", &json!({"locale":"en","key":"Unknown {thing}"})).unwrap(), "Unknown {thing}");
    let ar = call("i18n.t", &json!({"locale":"ar","key":"{activity} ({count})","args":{"activity":"x","count":2}})).unwrap();
    assert!(ar.as_str().unwrap().contains("\u{2068}2\u{2069}"));
    let fr = call("i18n.catalog", &json!({"locale":"fr_CA.UTF-8:en"})).unwrap();
    let mut keys = fr.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort();
    assert_eq!(keys, ["language", "locale", "rtl", "strings"]);
    assert_eq!((fr["locale"].as_str(), fr["language"].as_str(), fr["rtl"].as_bool()), (Some("fr-ca"), Some("fr"), Some(false)));
    assert_eq!(fr["strings"].as_object().unwrap().len(), 2027);
    assert_eq!(fr["strings"]["Close"], call("i18n.t", &json!({"locale":"fr","key":"Close"})).unwrap());
    assert_eq!(call("i18n.resolve", &json!({"locale":"zz:nb_NO"})).unwrap(), json!({"locale":"nb-no","language":"no","rtl":false}));
    assert_eq!(call("i18n.resolve", &json!({"locale":"ar_EG"})).unwrap()["rtl"], true);
    let languages = call("i18n.languages", &json!({"locale":"es"})).unwrap();
    assert_eq!(languages[0], json!({"id":"system","name":call("i18n.t", &json!({"locale":"es","key":"System language"})).unwrap()}));
    assert_eq!(languages.as_array().unwrap().len(), 21);
    assert_eq!(languages[14], json!({"id":"ar","name":"العربية"}));
    // ot3's strings for the domain's English sources exist in the catalog.
    let en = &t3_protocol::i18n::catalog()["en"];
    for source in ["{n} working", "{n} done", "{n} failed", "{n} stopped", "{n} idle", "{n} subagents", "Queued", "Running", "Waiting", "Idle · resumable", "Completed", "Failed", "Stopped", "Unknown"] {
        assert!(en.get(source).is_some(), "{source}");
    }
}
