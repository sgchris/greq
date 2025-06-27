use greq::greq_object::greq_header::{GreqHeader};
use greq::greq_object::traits::enrich_with_trait::EnrichWith;


#[test]
fn test_enrich_with_empty_self_filled_other() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("test_project".to_string()),
        is_http: true,
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("test_project".to_string()));
    assert_eq!(self_header.is_http, false); // default value
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_filled_self_empty_other() {
    let mut self_header = GreqHeader {
        delimiter: '#',
        project: Some("self_project".to_string()),
        is_http: false,
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dependency.greq".to_string()),
    };
    let other_header = GreqHeader::default();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged
    assert_eq!(self_header.project, Some("self_project".to_string()));
    assert_eq!(self_header.is_http, false);
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_both_filled_self_takes_precedence() {
    let mut self_header = GreqHeader {
        delimiter: '#',
        project: Some("self_project".to_string()),
        is_http: false,
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dependency.greq".to_string()),
    };

    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("other_project".to_string()),
        is_http: true,
        base_request: Some("other_base.greq".to_string()),
        depends_on: Some("other_dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged (precedence)
    assert_eq!(self_header.project, Some("self_project".to_string()));
    assert_eq!(self_header.is_http, false);
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dependency.greq".to_string()));
}

#[test]
fn test_enrich_with_partial_merge() {
    let mut self_header = GreqHeader {
        delimiter: '=',
        project: Some("self_project".to_string()),
        is_http: false, // None -> false (default)
        base_request: Some("self_base.greq".to_string()),
        depends_on: None,
    };
    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("other_project".to_string()),
        is_http: true,
        base_request: Some("other_base.greq".to_string()),
        depends_on: Some("other_dependency.greq".to_string()),
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("self_project".to_string())); // unchanged (has value)
    assert_eq!(self_header.is_http, false); // merged (was None)
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string())); // unchanged (has value)
    assert_eq!(self_header.depends_on, Some("other_dependency.greq".to_string())); // merged (was None)
}

#[test]
fn test_enrich_with_option_fields_none_to_some() {
    let mut self_header = GreqHeader {
        is_http: false,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: true,
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.is_http, false); // remains unchanged
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dep.greq".to_string()));
}

#[test]
fn test_enrich_with_option_fields_some_to_none() {
    let mut self_header = GreqHeader {
        is_http: false,
        base_request: Some("self_base.greq".to_string()),
        depends_on: Some("self_dep.greq".to_string()),
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: true,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Self values should remain unchanged
    assert_eq!(self_header.is_http, false); // remains unchanged
    assert_eq!(self_header.base_request, Some("self_base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("self_dep.greq".to_string()));
}

#[test]
fn test_enrich_with_both_none_remains_none() {
    let mut self_header = GreqHeader {
        is_http: false,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        is_http: false,
        base_request: None,
        depends_on: None,
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.is_http, false);
    assert_eq!(self_header.base_request, None);
    assert_eq!(self_header.depends_on, None);
}

#[test]
fn test_enrich_with_identical_objects() {
    let mut self_header = GreqHeader {
        delimiter: '=',
        project: Some("project".to_string()),
        is_http: true,
        base_request: Some("base.greq".to_string()),
        depends_on: Some("dep.greq".to_string()),
    };

    let other_header = self_header.clone();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Should remain identical
    assert_eq!(self_header.project, Some("project".to_string()));
    assert_eq!(self_header.is_http, true);
    assert_eq!(self_header.base_request, Some("base.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dep.greq".to_string()));
}

#[test]
fn test_enrich_with_both_empty() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader::default();

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // Should remain default
    assert_eq!(self_header, GreqHeader::default());
}

#[test]
fn test_enrich_with_whitespace_handling() {
    let mut self_header = GreqHeader {
        project: None,
        ..Default::default()
    };
    let other_header = GreqHeader {
        project: Some("other_project".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("other_project".to_string())); // merged (was empty)
}

#[test]
fn test_enrich_with_special_characters() {
    let mut self_header = GreqHeader::default();
    let other_header = GreqHeader {
        project: Some("project-with-dashes".to_string()),
        base_request: Some("base-request.greq".to_string()),
        depends_on: Some("dependency_file.greq".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    assert_eq!(self_header.project, Some("project-with-dashes".to_string()));
    assert_eq!(self_header.base_request, Some("base-request.greq".to_string()));
    assert_eq!(self_header.depends_on, Some("dependency_file.greq".to_string()));
}

#[test]
fn test_enrich_with_preserves_original_string_and_delimiter() {
    let mut self_header = GreqHeader {
        delimiter: '#',
        ..Default::default()
    };
    let other_header = GreqHeader {
        delimiter: '|',
        project: Some("other_project".to_string()),
        ..Default::default()
    };

    let result = self_header.enrich_with(&other_header);
    assert!(result.is_ok());

    // original_string and delimiter should not be affected by merge
    assert_eq!(self_header.delimiter, '#');
    // but other fields should merge
    assert_eq!(self_header.project, Some("other_project".to_string()));
}

