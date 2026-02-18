// 演示：从函数返回 Insert/BatchInsert/Update 构建器
#![cfg(feature = "db")]

use lsys_core::db::BatchInsert;
use lsys_core::db::Field;
use lsys_core::db::Insert;
use lsys_core::db::TableMeta;
use lsys_core::db::TableName;
use lsys_core::db::Update;

#[test]

fn builder_return_demo() {
    struct UserModel;
    impl TableMeta for UserModel {
        fn table_name() -> TableName {
            TableName::new("users")
        }
    }
    impl UserModel {
        const ID: Field<u64> = Field::new("id");
        const NAME: Field<String> = Field::new("name");
        const AGE: Field<i32> = Field::new("age");
        const STATUS: Field<i8> = Field::new("status");
        const SCORE: Field<f64> = Field::new("score");
    }

    // ✅ 示例 1：返回 Insert<'static>
    fn create_insert() -> Insert<'static, UserModel> {
        Insert::<UserModel>::new()
            .set(UserModel::NAME, "张三".to_string())
            .set(UserModel::AGE, 25i32)
            .set(UserModel::STATUS, 1i8)
            .set(UserModel::SCORE, 98.5f64)
    }

    // ✅ 示例 2：返回 Update<'static>
    fn create_update() -> Update<'static, UserModel> {
        Update::<UserModel>::new()
            .set(UserModel::NAME, "李四".to_string())
            .set(UserModel::AGE, 30)
    }

    // ✅ 示例 3：返回 BatchInsert<'static>
    fn create_batch() -> BatchInsert<'static, UserModel> {
        let mut batch = BatchInsert::<UserModel>::with_capacity(3);
        for i in 1..=3 {
            batch = batch.push(
                Insert::<UserModel>::new()
                    .set(UserModel::ID, i as u64)
                    .set(UserModel::NAME, format!("用户_{}", i))
                    .set(UserModel::AGE, 20 + i)
                    .set(UserModel::STATUS, 1i8),
            );
        }
        batch
    }

    println!("🎯 演示：从函数返回构建器\n");
    println!("{}", "=".repeat(50));

    println!("\n📝 示例 1: Insert<'static>");
    let _insert = create_insert();
    println!("✅ Insert 创建成功\n");

    println!("📝 示例 2: Update<'static>");
    let _update = create_update();
    println!("✅ Update 创建成功\n");

    println!("📝 示例 3: BatchInsert<'static>");
    let _batch = create_batch();
    println!("✅ BatchInsert 创建成功 (3条记录)\n");

    println!("{}", "=".repeat(50));
    println!("\n✨ 关键点：");
    println!("  • 所有字段都是 owned 类型 → Insert<'static>");
    println!("  • 可以安全地从函数返回");
    println!("  • 支持链式调用和动态组装\n");

    println!("🎉 完成！");

    // ============== 验证：String 不会自动转为引用 ==============
    println!("\n{}", "=".repeat(50));
    println!("🔬 验证：String vs &str 的类型推断\n");

    // 测试：明确使用 String（owned）
    fn returns_static() -> Insert<'static, UserModel> {
        let owned_string = String::from("这是owned String");

        // ✅ String 精确匹配 impl for String，存储 owned 值
        Insert::<UserModel>::new()
            .set(UserModel::NAME, owned_string) // 传入 String
            .set(UserModel::AGE, 30)
        // owned_string 被 move 进去，不是借用
    }

    // 测试：明确使用引用
    fn returns_with_lifetime<'a>(name: &'a str) -> Insert<'a, UserModel> {
        // &str 精确匹配 impl for &'a str，存储引用
        Insert::<UserModel>::new()
            .set(UserModel::NAME, name) // 传入 &str，生命周期为 'a
            .set(UserModel::AGE, 25)
    }

    // 验证 'static 可以返回
    let _insert_static = returns_static();
    println!("✅ String → Insert<'static> 可以从函数返回");

    // 验证引用版本需要生命周期
    let name = String::from("引用测试");
    let _insert_ref = returns_with_lifetime(&name);
    println!("✅ &str → Insert<'a> 绑定到外部生命周期");

    // 关键测试：String 不会被隐式转为 &str
    fn test_no_implicit_conversion() -> Insert<'static, UserModel> {
        let s1 = String::from("字符串1");

        // 这两个都是 String（owned），不会被转为 &str
        Insert::<UserModel>::new()
            .set(UserModel::NAME, s1) // String，不是 &str
            .set(UserModel::STATUS, 1i8)
        // s1 被 move，如果是 &str 这里会编译失败
    }

    let _ = test_no_implicit_conversion();
    println!("✅ String 不会隐式转换为 &str");

    // 对比：如果用引用，必须保证生命周期
    fn test_reference_needs_lifetime<'a>(s: &'a String) -> Insert<'a, UserModel> {
        Insert::<UserModel>::new()
            .set(UserModel::NAME, s) // &String → 存储 &str，需要 'a
            .set(UserModel::STATUS, 1i8)
    }

    let owned = String::from("测试");
    let _ = test_reference_needs_lifetime(&owned);
    println!("✅ &String 需要生命周期绑定");

    println!("\n📌 结论：Rust 类型推断是精确匹配的");
    println!("   • String → 匹配 impl for String → 存储 owned → 'static");
    println!("   • &str → 匹配 impl for &str → 存储引用 → 'a");
    println!("   • &String → 匹配 impl for &String → 存储引用 → 'a");
    println!("   • 不会发生自动转换！\n");
}
