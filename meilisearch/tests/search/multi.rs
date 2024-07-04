use meili_snap::{json_string, snapshot};

use super::{DOCUMENTS, FRUITS_DOCUMENTS, NESTED_DOCUMENTS};
use crate::common::Server;
use crate::json;
use crate::search::SCORE_DOCUMENTS;

#[actix_rt::test]
async fn search_empty_list() {
    let server = Server::new().await;

    let (response, code) = server.multi_search(json!({"queries": []})).await;
    snapshot!(code, @"200 OK");
    snapshot!(json_string!(response), @r###"
    {
      "results": []
    }
    "###);
}

#[actix_rt::test]
async fn federation_empty_list() {
    let server = Server::new().await;

    let (response, code) = server.multi_search(json!({"federation": {}, "queries": []})).await;
    snapshot!(code, @"200 OK");
    snapshot!(json_string!(response, {".processingTimeMs" => "[time]"}), @r###"
    {
      "hits": [],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 0
    }
    "###);
}

#[actix_rt::test]
async fn search_json_object() {
    let server = Server::new().await;

    let (response, code) = server.multi_search(json!({})).await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Missing field `queries`",
      "code": "bad_request",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#bad_request"
    }
    "###);
}

#[actix_rt::test]
async fn federation_no_queries() {
    let server = Server::new().await;

    let (response, code) = server.multi_search(json!({"federation": {}})).await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Missing field `queries`",
      "code": "bad_request",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#bad_request"
    }
    "###);
}

#[actix_rt::test]
async fn search_json_array() {
    let server = Server::new().await;

    let (response, code) = server.multi_search(json!([])).await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Invalid value type: expected an object, but found an array: `[]`",
      "code": "bad_request",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#bad_request"
    }
    "###);
}

#[actix_rt::test]
async fn simple_search_single_index() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "test", "q": "captain"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response["results"], { "[].processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    [
      {
        "indexUid": "test",
        "hits": [
          {
            "title": "Gläss",
            "id": "450465",
            "_vectors": {
              "manual": [
                -100.0,
                340.0,
                90.0
              ]
            }
          }
        ],
        "query": "glass",
        "processingTimeMs": "[time]",
        "limit": 20,
        "offset": 0,
        "estimatedTotalHits": 1
      },
      {
        "indexUid": "test",
        "hits": [
          {
            "title": "Captain Marvel",
            "id": "299537",
            "_vectors": {
              "manual": [
                1.0,
                2.0,
                54.0
              ]
            }
          }
        ],
        "query": "captain",
        "processingTimeMs": "[time]",
        "limit": 20,
        "offset": 0,
        "estimatedTotalHits": 1
      }
    ]
    "###);
}

#[actix_rt::test]
async fn federation_single_search_single_index() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response, { ".processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    {
      "hits": [
        {
          "title": "Gläss",
          "id": "450465",
          "_vectors": {
            "manual": [
              -100.0,
              340.0,
              90.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 0,
            "weightedRankingScore": 1.0
          }
        }
      ],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 1
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_search_single_index() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = SCORE_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid": "test", "q": "the bat"},
        {"indexUid": "test", "q": "badman returns"},
        {"indexUid" : "test", "q": "batman"},
        {"indexUid": "test", "q": "batman returns"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response, { ".processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    {
      "hits": [
        {
          "title": "Batman",
          "id": "D",
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 2,
            "weightedRankingScore": 1.0
          }
        },
        {
          "title": "Batman Returns",
          "id": "C",
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 3,
            "weightedRankingScore": 1.0
          }
        },
        {
          "title": "Batman the dark knight returns: Part 1",
          "id": "A",
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 2,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "title": "Batman the dark knight returns: Part 2",
          "id": "B",
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 2,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "title": "Badman",
          "id": "E",
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 1,
            "weightedRankingScore": 0.5
          }
        }
      ],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 5
    }
    "###);
}

#[actix_rt::test]
async fn federation_two_search_single_index() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "test", "q": "captain"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response, { ".processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    {
      "hits": [
        {
          "title": "Gläss",
          "id": "450465",
          "_vectors": {
            "manual": [
              -100.0,
              340.0,
              90.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 0,
            "weightedRankingScore": 1.0
          }
        },
        {
          "title": "Captain Marvel",
          "id": "299537",
          "_vectors": {
            "manual": [
              1.0,
              2.0,
              54.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 1,
            "weightedRankingScore": 0.9848484848484848
          }
        }
      ],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 2
    }
    "###);
}

#[actix_rt::test]
async fn simple_search_missing_index_uid() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"q": "glass"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    insta::assert_json_snapshot!(response, @r###"
    {
      "message": "Missing field `indexUid` inside `.queries[0]`",
      "code": "missing_index_uid",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#missing_index_uid"
    }
    "###);
}

#[actix_rt::test]
async fn federation_simple_search_missing_index_uid() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"q": "glass"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    insta::assert_json_snapshot!(response, @r###"
    {
      "message": "Missing field `indexUid` inside `.queries[0]`",
      "code": "missing_index_uid",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#missing_index_uid"
    }
    "###);
}

#[actix_rt::test]
async fn simple_search_illegal_index_uid() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid": "hé", "q": "glass"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    insta::assert_json_snapshot!(response, @r###"
    {
      "message": "Invalid value at `.queries[0].indexUid`: `hé` is not a valid index uid. Index uid can be an integer or a string containing only alphanumeric characters, hyphens (-) and underscores (_).",
      "code": "invalid_index_uid",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_index_uid"
    }
    "###);
}

#[actix_rt::test]
async fn federation_search_illegal_index_uid() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid": "hé", "q": "glass"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    insta::assert_json_snapshot!(response, @r###"
    {
      "message": "Invalid value at `.queries[0].indexUid`: `hé` is not a valid index uid. Index uid can be an integer or a string containing only alphanumeric characters, hyphens (-) and underscores (_).",
      "code": "invalid_index_uid",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_index_uid"
    }
    "###);
}

#[actix_rt::test]
async fn simple_search_two_indexes() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response["results"], { "[].processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    [
      {
        "indexUid": "test",
        "hits": [
          {
            "title": "Gläss",
            "id": "450465",
            "_vectors": {
              "manual": [
                -100.0,
                340.0,
                90.0
              ]
            }
          }
        ],
        "query": "glass",
        "processingTimeMs": "[time]",
        "limit": 20,
        "offset": 0,
        "estimatedTotalHits": 1
      },
      {
        "indexUid": "nested",
        "hits": [
          {
            "id": 852,
            "father": "jean",
            "mother": "michelle",
            "doggos": [
              {
                "name": "bobby",
                "age": 2
              },
              {
                "name": "buddy",
                "age": 4
              }
            ],
            "cattos": "pésti",
            "_vectors": {
              "manual": [
                1.0,
                2.0,
                3.0
              ]
            }
          },
          {
            "id": 654,
            "father": "pierre",
            "mother": "sabine",
            "doggos": [
              {
                "name": "gros bill",
                "age": 8
              }
            ],
            "cattos": [
              "simba",
              "pestiféré"
            ],
            "_vectors": {
              "manual": [
                1.0,
                2.0,
                54.0
              ]
            }
          }
        ],
        "query": "pésti",
        "processingTimeMs": "[time]",
        "limit": 20,
        "offset": 0,
        "estimatedTotalHits": 2
      }
    ]
    "###);
}

#[actix_rt::test]
async fn federation_two_search_two_indexes() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response, { ".processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    {
      "hits": [
        {
          "title": "Gläss",
          "id": "450465",
          "_vectors": {
            "manual": [
              -100.0,
              340.0,
              90.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 0,
            "weightedRankingScore": 1.0
          }
        },
        {
          "id": 852,
          "father": "jean",
          "mother": "michelle",
          "doggos": [
            {
              "name": "bobby",
              "age": 2
            },
            {
              "name": "buddy",
              "age": 4
            }
          ],
          "cattos": "pésti",
          "_vectors": {
            "manual": [
              1.0,
              2.0,
              3.0
            ]
          },
          "_federation": {
            "indexUid": "nested",
            "sourceQuery": 1,
            "weightedRankingScore": 1.0
          }
        },
        {
          "id": 654,
          "father": "pierre",
          "mother": "sabine",
          "doggos": [
            {
              "name": "gros bill",
              "age": 8
            }
          ],
          "cattos": [
            "simba",
            "pestiféré"
          ],
          "_vectors": {
            "manual": [
              1.0,
              2.0,
              54.0
            ]
          },
          "_federation": {
            "indexUid": "nested",
            "sourceQuery": 1,
            "weightedRankingScore": 0.7803030303030303
          }
        }
      ],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 3
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_search_multiple_indexes() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let index = server.index("score");
    let documents = SCORE_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(2).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid" : "test", "q": "captain"},
        {"indexUid": "nested", "q": "pésti"},
        {"indexUid" : "test", "q": "Escape"},
        {"indexUid": "nested", "q": "jean"},
        {"indexUid": "score", "q": "jean"},
        {"indexUid": "test", "q": "the bat"},
        {"indexUid": "score", "q": "the bat"},
        {"indexUid": "score", "q": "badman returns"},
        {"indexUid" : "score", "q": "batman"},
        {"indexUid": "score", "q": "batman returns"},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response, { ".processingTimeMs" => "[time]", ".**._rankingScore" => "[score]" }, @r###"
    {
      "hits": [
        {
          "title": "Gläss",
          "id": "450465",
          "_vectors": {
            "manual": [
              -100.0,
              340.0,
              90.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 0,
            "weightedRankingScore": 1.0
          }
        },
        {
          "id": 852,
          "father": "jean",
          "mother": "michelle",
          "doggos": [
            {
              "name": "bobby",
              "age": 2
            },
            {
              "name": "buddy",
              "age": 4
            }
          ],
          "cattos": "pésti",
          "_vectors": {
            "manual": [
              1.0,
              2.0,
              3.0
            ]
          },
          "_federation": {
            "indexUid": "nested",
            "sourceQuery": 2,
            "weightedRankingScore": 1.0
          }
        },
        {
          "title": "Batman",
          "id": "D",
          "_federation": {
            "indexUid": "score",
            "sourceQuery": 9,
            "weightedRankingScore": 1.0
          }
        },
        {
          "title": "Batman Returns",
          "id": "C",
          "_federation": {
            "indexUid": "score",
            "sourceQuery": 10,
            "weightedRankingScore": 1.0
          }
        },
        {
          "title": "Captain Marvel",
          "id": "299537",
          "_vectors": {
            "manual": [
              1.0,
              2.0,
              54.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 1,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "title": "Escape Room",
          "id": "522681",
          "_vectors": {
            "manual": [
              10.0,
              -23.0,
              32.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 3,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "id": 951,
          "father": "jean-baptiste",
          "mother": "sophie",
          "doggos": [
            {
              "name": "turbo",
              "age": 5
            },
            {
              "name": "fast",
              "age": 6
            }
          ],
          "cattos": [
            "moumoute",
            "gomez"
          ],
          "_vectors": {
            "manual": [
              10.0,
              23.0,
              32.0
            ]
          },
          "_federation": {
            "indexUid": "nested",
            "sourceQuery": 4,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "title": "Batman the dark knight returns: Part 1",
          "id": "A",
          "_federation": {
            "indexUid": "score",
            "sourceQuery": 9,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "title": "Batman the dark knight returns: Part 2",
          "id": "B",
          "_federation": {
            "indexUid": "score",
            "sourceQuery": 9,
            "weightedRankingScore": 0.9848484848484848
          }
        },
        {
          "id": 654,
          "father": "pierre",
          "mother": "sabine",
          "doggos": [
            {
              "name": "gros bill",
              "age": 8
            }
          ],
          "cattos": [
            "simba",
            "pestiféré"
          ],
          "_vectors": {
            "manual": [
              1.0,
              2.0,
              54.0
            ]
          },
          "_federation": {
            "indexUid": "nested",
            "sourceQuery": 2,
            "weightedRankingScore": 0.7803030303030303
          }
        },
        {
          "title": "Badman",
          "id": "E",
          "_federation": {
            "indexUid": "score",
            "sourceQuery": 8,
            "weightedRankingScore": 0.5
          }
        },
        {
          "title": "How to Train Your Dragon: The Hidden World",
          "id": "166428",
          "_vectors": {
            "manual": [
              -100.0,
              231.0,
              32.0
            ]
          },
          "_federation": {
            "indexUid": "test",
            "sourceQuery": 6,
            "weightedRankingScore": 0.4166666666666667
          }
        }
      ],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 12
    }
    "###);
}

#[actix_rt::test]
async fn search_one_index_doesnt_exist() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Index `nested` not found.",
      "code": "index_not_found",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#index_not_found"
    }
    "###);
}

#[actix_rt::test]
async fn federation_one_index_doesnt_exist() {
    let server = Server::new().await;
    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Index `nested` not found.",
      "code": "index_not_found",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#index_not_found"
    }
    "###);
}

#[actix_rt::test]
async fn search_multiple_indexes_dont_exist() {
    let server = Server::new().await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[0]`: Index `test` not found.",
      "code": "index_not_found",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#index_not_found"
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_indexes_dont_exist() {
    let server = Server::new().await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    // order of indexes that are not found depends on the alphabetical order of index names
    // the query index is the lowest index with that index
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Index `nested` not found.",
      "code": "index_not_found",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#index_not_found"
    }
    "###);
}

#[actix_rt::test]
async fn search_one_query_error() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass", "facets": ["title"]},
        {"indexUid": "nested", "q": "pésti"},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[0]`: Invalid facet distribution, this index does not have configured filterable attributes.",
      "code": "invalid_search_facets",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_facets"
    }
    "###);
}

#[actix_rt::test]
async fn federation_one_query_error() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti", "filter": ["title = toto"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Attribute `title` is not filterable. This index does not have configured filterable attributes.\n1:6 title = toto",
      "code": "invalid_search_filter",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_filter"
    }
    "###);
}

#[actix_rt::test]
async fn federation_one_query_sort_error() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti", "sort": ["doggos:desc"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Attribute `doggos` is not sortable. This index does not have configured sortable attributes.",
      "code": "invalid_search_sort",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_sort"
    }
    "###);
}

#[actix_rt::test]
async fn search_multiple_query_errors() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass", "facets": ["title"]},
        {"indexUid": "nested", "q": "pésti", "facets": ["doggos"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[0]`: Invalid facet distribution, this index does not have configured filterable attributes.",
      "code": "invalid_search_facets",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_facets"
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_query_errors() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass", "filter": ["title = toto"]},
        {"indexUid": "nested", "q": "pésti", "filter": ["doggos IN [intel, kefir]"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[0]`: Attribute `title` is not filterable. This index does not have configured filterable attributes.\n1:6 title = toto",
      "code": "invalid_search_filter",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_filter"
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_query_sort_errors() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass", "sort": ["title:desc"]},
        {"indexUid": "nested", "q": "pésti", "sort": ["doggos:desc"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[0]`: Attribute `title` is not sortable. This index does not have configured sortable attributes.",
      "code": "invalid_search_sort",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_sort"
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_query_errors_interleaved() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti", "filter": ["doggos IN [intel, kefir]"]},
        {"indexUid" : "test", "q": "glass", "filter": ["title = toto"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Attribute `doggos` is not filterable. This index does not have configured filterable attributes.\n1:7 doggos IN [intel, kefir]",
      "code": "invalid_search_filter",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_filter"
    }
    "###);
}

#[actix_rt::test]
async fn federation_multiple_query_sort_errors_interleaved() {
    let server = Server::new().await;

    let index = server.index("test");

    let documents = DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(0).await;

    let index = server.index("nested");
    let documents = NESTED_DOCUMENTS.clone();
    index.add_documents(documents, None).await;
    index.wait_task(1).await;

    let (response, code) = server
        .multi_search(json!({"queries": [
        {"indexUid" : "test", "q": "glass"},
        {"indexUid": "nested", "q": "pésti", "sort": ["doggos:desc"]},
        {"indexUid" : "test", "q": "glass", "sort": ["title:desc"]},
        ]}))
        .await;
    snapshot!(code, @"400 Bad Request");
    snapshot!(json_string!(response), @r###"
    {
      "message": "Inside `.queries[1]`: Attribute `doggos` is not sortable. This index does not have configured sortable attributes.",
      "code": "invalid_search_sort",
      "type": "invalid_request",
      "link": "https://docs.meilisearch.com/errors#invalid_search_sort"
    }
    "###);
}

#[actix_rt::test]
async fn federation_filter() {
    let server = Server::new().await;

    let index = server.index("fruits");

    let documents = FRUITS_DOCUMENTS.clone();
    let (value, _) = index.add_documents(documents, None).await;
    index.wait_task(value.uid()).await;

    let (value, _) = index
        .update_settings(
            json!({"searchableAttributes": ["name"], "filterableAttributes": ["BOOST"]}),
        )
        .await;
    index.wait_task(value.uid()).await;

    let (response, code) = server
        .multi_search(json!({"federation": {}, "queries": [
        {"indexUid" : "fruits", "q": "apple red", "filter": "BOOST = true", "showRankingScore": true, "federated": {"weight": 3.0}},
        {"indexUid": "fruits", "q": "apple red", "showRankingScore": true},
        ]}))
        .await;
    snapshot!(code, @"200 OK");
    insta::assert_json_snapshot!(response, { ".processingTimeMs" => "[time]" }, @r###"
    {
      "hits": [
        {
          "name": "Exclusive sale: Red delicious apple",
          "id": "red-delicious-boosted",
          "BOOST": true,
          "_federation": {
            "indexUid": "fruits",
            "sourceQuery": 0,
            "weightedRankingScore": 2.7281746031746033
          },
          "_rankingScore": 0.9093915343915344
        },
        {
          "name": "Exclusive sale: green apple",
          "id": "green-apple-boosted",
          "BOOST": true,
          "_federation": {
            "indexUid": "fruits",
            "sourceQuery": 0,
            "weightedRankingScore": 1.318181818181818
          },
          "_rankingScore": 0.4393939393939394
        },
        {
          "name": "Red apple gala",
          "id": "red-apple-gala",
          "_federation": {
            "indexUid": "fruits",
            "sourceQuery": 1,
            "weightedRankingScore": 0.953042328042328
          },
          "_rankingScore": 0.953042328042328
        }
      ],
      "processingTimeMs": "[time]",
      "limit": 20,
      "offset": 0,
      "estimatedTotalHits": 3
    }
    "###);
}
