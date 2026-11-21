//! 针对 Apple 美国站真实接口的契约测试。默认不参与 `cargo test`，需要显式启用：
//!
//! ```shell
//! cargo test -p apw-core --features live --test live -- --nocapture
//! ```
//!
//! 它存在的理由很直接：上游项目就是因为 Apple 悄悄换掉了接口而彻底失效，却因为
//! 没有任何测试，半年时间里没人发现程序返回的「无货」其实是被拦截。单元测试用
//! 假响应验证解析逻辑，但拦不住这种事；只有真的打一次 Apple 才行。
//!
//! 本修改版以美国站为主，用例全部使用 `en_US`，覆盖 iPhone 与 Apple HomeView。
//! 建议手动触发，而不是每次提交都跑，以免给 Apple 添麻烦。

#![cfg(feature = "live")]

use apw_core::apple::{ApiError, AppleClient, ClientConfig};
use apw_core::catalog::Catalog;
use apw_core::model::{Availability, Category, UnknownReason, region_by_locale};

const LOCALE: &str = "en_US";

/// 美国 Austin 的 Barton Creek 店。2026-11-20 实测可正常返回 iPhone 与 HomeView。
const STORE: &str = "R085";

/// iPhone 18 Pro Max 1TB Black。零件号随机型更新失效时更新这里，而不是删掉测试。
const IPHONE: &str = "MJWD4LL/A";

/// iPhone Duo 256GB Night Sky。
const DUO: &str = "MK1F4LL/A";

/// Apple HomeView Silver。
const HOMEVIEW: &str = "MDH14LL/A";

fn client() -> AppleClient {
    // 使用正式客户端配置，确保真实接口测试覆盖应用实际采用的请求节奏。
    AppleClient::new(ClientConfig::default()).expect("构造客户端失败")
}

#[tokio::test(flavor = "multi_thread")]
async fn 美国站取货接口还能用() {
    let client = client();
    let region = region_by_locale(LOCALE).expect("地区表里没有美国站");
    let parts = vec![IPHONE.to_string()];

    let result = client
        .pickup_message(region, STORE, &parts)
        .await
        .unwrap_or_else(|e| match e {
            ApiError::Blocked(d) => {
                panic!("请求被 Apple 拦截，接口或请求特征需要调整：{d}")
            }
            ApiError::SchemaDrift { field, raw } => {
                panic!("响应结构已改变，解析逻辑需要更新：{field} = {raw}")
            }
            other => panic!("查询失败：{other}"),
        });

    assert_eq!(result.store_number, STORE, "返回了别的门店");
    let status = result
        .parts
        .get(IPHONE)
        .unwrap_or_else(|| panic!("响应里没有零件号 {IPHONE}（可能已下架，需更新用例）"));

    assert!(
        !status.pickup_display.is_empty(),
        "pickupDisplay 为空，字段名可能已变更"
    );

    // 是有货还是无货取决于当下库存，不做断言；但解析结果不能是「无法识别」——
    // 那说明 Apple 换了词表，得去补 availability_from 的分支。
    if let Availability::Unknown(UnknownReason::SchemaDrift { raw, .. }) = &status.availability {
        panic!("返回了无法识别的 pickupDisplay {raw:?}，需要补充分支");
    }

    println!(
        "{LOCALE} {:12} {IPHONE:12} -> {:4} (pickupDisplay={}, 商品名={})",
        result.store_name,
        status.availability.label(),
        status.pickup_display,
        status.product_title.as_deref().unwrap_or("(无)"),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn 必须以http2跟apple说话() {
    // 这条测试守的是一个「配置写漏了、功能却完全没坏」的缺陷。
    //
    // reqwest 的 HTTP/2 支持挂在 http2 feature 上，而我们用的是
    // default-features = false。那个 feature 曾经漏了整整一个版本：客户端静默
    // 退回 HTTP/1.1，查询照常成功、离线测试全绿，谁都看不出问题。但我们的 UA
    // 自称 Chrome —— 真实的 Chrome 早就不用 HTTP/1.1 跟 apple.com 说话了，
    // 这个矛盾是最一眼可辨的机器人特征，在受风控审查的网络上直接换来 HTTP 541。
    let client = client();
    let region = region_by_locale(LOCALE).expect("地区表里没有美国站");
    let version = client
        .negotiated_http_version(region)
        .await
        .unwrap_or_else(|e| panic!("探测 HTTP 版本失败：{e}"));

    assert!(
        version.contains('2'),
        "协商出来的是 {version}，不是 HTTP/2 —— \
         检查 Cargo.toml 里 reqwest 的 http2 feature 是不是又漏了"
    );
    println!("{LOCALE} 协商出的协议：{version}");
}

#[tokio::test(flavor = "multi_thread")]
async fn 查询时必须带上cookie() {
    // Apple 的边缘节点会对没带 cookie 的取货查询下手。这条测试不去验「带了
    // cookie 就不会被拦」（在没被盯上的网络里验不出来），只钉住一件能验的事：
    // **罐子里真的有东西**。一个「以为自己在带 cookie、其实罐是空的」的客户端，
    // 功能上和现在一模一样，不会有任何迹象。
    let client = client();
    let region = region_by_locale(LOCALE).expect("地区表里没有美国站");

    assert!(
        client.cookies_for(region).is_none(),
        "还没发过任何请求，cookie 罐就该是空的"
    );

    let parts = vec![IPHONE.to_string()];
    let result = client.pickup_message(region, STORE, &parts).await;

    let cookies = client
        .cookies_for(region)
        .expect("查过一次之后，cookie 罐不该还是空的 —— 暖场或响应里的 Set-Cookie 没生效");

    // Cookie 是会话凭据，只打印名称，不把值写进测试日志。
    let cookie_names: Vec<_> = cookies
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('=').map(|(name, _)| name))
        .collect();
    println!("攒到的 Cookie 名称：{cookie_names:?}");
    // Apple 的 shop 会话 cookie。名字变了要来更新这里，而不是删掉断言。
    assert!(
        cookies.contains("dssid2") || cookies.contains("as_dc"),
        "攒到的 cookie 里没有 shop 的会话项，只有：{cookie_names:?}"
    );

    result.unwrap_or_else(|e| panic!("已成功建立 Cookie 会话，但库存查询失败：{e}"));
}

#[tokio::test(flavor = "multi_thread")]
async fn 无效零件号不会被判成无货() {
    // 本项目最关键的不变量在真实接口上也必须成立。
    let client = client();
    let region = region_by_locale(LOCALE).unwrap();

    match client
        .pickup_message(region, STORE, &["NOSUCHPART/A".to_string()])
        .await
    {
        // Apple 对无效零件号会直接报业务错误，这是可接受且正确的结果。
        Err(e) => println!("无效零件号返回错误（可接受）：{e}"),
        Ok(result) => {
            if let Some(status) = result.parts.get("NOSUCHPART/A") {
                assert_ne!(
                    status.availability,
                    Availability::OutOfStock,
                    "无效零件号被判定成了「无货」，这正是上游那个致命缺陷：{status:?}"
                );
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn 美国站每个品类的零件号接口都认() {
    // 「HomeView 也能用同一个取货接口查」是 Home 品类的前提。一旦美国站对
    // HomeView 回 ineligible（例如又带上了 pl=true），界面上会是一屏「无货」。
    //
    // 零件号从内嵌目录里现取，不写死：写死的表会随机型更新失效。
    let client = client();
    let region = region_by_locale(LOCALE).expect("地区表里没有美国站");
    let catalog = Catalog::new();
    let products = catalog.products(LOCALE).expect("内嵌目录应当可用");

    for category in Category::ALL {
        if region.families_in(*category).next().is_none() {
            continue;
        }
        let product = products
            .iter()
            .find(|p| p.category == *category)
            .unwrap_or_else(|| panic!("内嵌目录里没有 {} 商品", category.title()));
        let parts = vec![product.part_number.clone()];

        let result = client
            .pickup_message(region, STORE, &parts)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "{} 的零件号 {} 查询失败：{e}",
                    category.title(),
                    product.part_number
                )
            });

        let status = result.parts.get(&product.part_number).unwrap_or_else(|| {
            panic!(
                "{} 的响应里没有零件号 {}（接口可能不接受这个品类）",
                category.title(),
                product.part_number
            )
        });

        // 有货没货取决于当下库存，不做断言。要守住的是「必须拿到明确答复」。
        assert!(
            !status.availability.is_unknown(),
            "{} 的 {} 没拿到明确答复：{:?}",
            category.title(),
            product.part_number,
            status.availability
        );
        // ineligible 只代表「这家店不支持取这个型号」。美国站全部门店都卖
        // HomeView，回 ineligible 说明请求参数又出了问题。
        assert_ne!(
            status.pickup_display,
            "ineligible",
            "{} 的 {} 被判为不支持取货，检查美国站取货查询参数（pl=true）",
            category.title(),
            product.part_number
        );

        println!(
            "{:11} {:12} -> {:4} （{}，pickupDisplay={}）",
            category.title(),
            product.part_number,
            status.availability.label(),
            product.title,
            status.pickup_display
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn 一次请求可以带多个零件号() {
    // 这决定了正确的请求策略：每门店每轮只发一个请求，覆盖该店所有关注型号。
    // 故意混合 iPhone 与 HomeView：同一家店的两个品类会合并在同一个请求里。
    let client = client();
    let region = region_by_locale(LOCALE).unwrap();
    let parts: Vec<String> = [IPHONE, DUO, HOMEVIEW]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let result = client
        .pickup_message(region, STORE, &parts)
        .await
        .expect("批量查询应当成功");

    assert_eq!(
        result.parts.len(),
        parts.len(),
        "批量请求的型号数与返回的不一致：{:?}",
        result.parts.keys().collect::<Vec<_>>()
    );
    for p in &parts {
        assert!(result.parts.contains_key(p), "响应里缺少 {p}");
    }
}
