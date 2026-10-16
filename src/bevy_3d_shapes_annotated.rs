//! ============================================================================
//! Bevy 3D Shapes - 詳細インライン解説付きソースコード (bevy_3d_shapes_annotated.rs)
//! ============================================================================
//! 本ファイルは `bevy_3d_shapes/src/main.rs` の各行に対して、
//! 使用されている API・Rust特有の構文・設計上の意図を網羅的に解説した学習用コードです。
//! ----------------------------------------------------------------------------

// 【モジュール・トレイトのインポート】
use std::f32::consts::PI; // 円周率 π (3.1415926...)。回転角度の計算（ラジアン）で使用します。

// 【条件付きコンパイル】Web(WASM)以外のネイティブビルドでのみワイヤーフレーム用モジュールをインポート
#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};

// Bevyエンジンの主要型・モジュールをインポート
use bevy::{
    asset::RenderAssetUsages, // アセットをGPUメモリ/CPUメモリのどちらに置くか指定する設定値
    color::palettes::basic::SILVER, // 基本カラーパレットから「シルバー」色を取得
    input::common_conditions::{input_just_pressed, input_toggle_active}, // システムの実行条件（キー入力判定など）
    prelude::*, // Bevyで頻繁に使う型（App, Commands, Entity, Query, Transform, Mesh など）を一括インポート
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat}, // テクスチャ生成用のサイズ・次元・フォーマット設定
        settings::{InstanceFlags, RenderCreation, WgpuSettings}, // 低レイヤーのレンダリング・GPU設定
        RenderPlugin, // レンダリングプラグイン本体
    },
};

// ============================================================================
// 1. main 関数：アプリケーションの構築とシステム登録
// ============================================================================
fn main() {
    // App::new() でBevyアプリケーションのビルダーインスタンスを生成
    App::new()
        // .add_plugins(...) で標準機能グループ（DefaultPlugins）を拡張設定付きで追加
        .add_plugins((
            DefaultPlugins
                // 画像フィルタリングの設定：ドット絵風（Nearest Neighbor）にしてデバッグテクスチャをクッキリ表示
                .set(ImagePlugin::default_nearest())
                // レンダリング設定のオーバーライド（グラフィックカードの互換性フラグを設定）
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        instance_flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                        ..default() // その他の設定はデフォルト値を継承 (Struct Update Syntax)
                    })),
                    ..default()
                }),
            // Web(WASM)環境以外でのみワイヤーフレーム描画用プラグインを追加
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin::default(),
        ))
        // 【Startup スケジュール】アプリ起動時に1回だけ実行する「setup」システムを登録
        .add_systems(Startup, setup)
        // 【Update スケジュール】毎フレーム実行されるシステム群を条件付きで登録
        .add_systems(
            Update,
            (
                // Rキーが押されるたびに回転（rotate）システムのON/OFFを切り替え（初期状態はtrue=回転中）
                rotate.run_if(input_toggle_active(true, KeyCode::KeyR)),
                // Tabキーが押された瞬間だけ「advance_rows」システム（奥行き切り替え）を実行
                advance_rows.run_if(input_just_pressed(KeyCode::Tab)),
                // Web環境以外でのみ、Spaceキーによるワイヤーフレーム切り替えシステムを実行
                #[cfg(not(target_arch = "wasm32"))]
                toggle_wireframe,
            ),
        )
        // アプリケーションのメインループを開始（イベントループに入り、ゲームウィンドウを起動）
        .run();
}

// ============================================================================
// 2. マーカーコンポーネント & 定数定義
// ============================================================================

/// マーカーコンポーネント：3Dオブジェクト（Shape）であることを目印として付与する空構造体
/// `#[derive(Component)]` を付けることで、Bevy ECSのコンポーネントとして登録されます。
#[derive(Component)]
struct Shape;

// 3Dオブジェクトを横一列に並べる際の位置計算用定数（単位：Bevyワールド単位/メートル相当）
const SHAPES_X_EXTENT: f32 = 14.0;    // 3D基本形状の横幅の配置範囲
const EXTRUSION_X_EXTENT: f32 = 14.0; // 押し出し形状の横幅の配置範囲
const Z_EXTENT: f32 = 8.0;          // 行間（前・中・後）のZ方向の距離
const THICKNESS: f32 = 0.1;         // リング形状の中空化処理における厚み

// ============================================================================
// 3. setup 関数：ワールドの初期化（アセット登録 & Entityの生成）
// ============================================================================
fn setup(
    mut commands: Commands,                  // ワールドに対する非同期操作（Entityの生成・削除）を行うコマンドバッファ
    mut meshes: ResMut<Assets<Mesh>>,        // 3Dメッシュアセットの可変参照（新しいメッシュの追加・編集が可能）
    mut images: ResMut<Assets<Image>>,       // 画像テクスチャアセットの可変参照
    mut materials: ResMut<Assets<StandardMaterial>>, // PBRマテリアル（質感）アセットの可変参照
) {
    // --- 1. マテリアル（質感）の生成 ---
    // 自作のデバッグテクスチャ画像（uv_debug_texture）を生成し、アセット管理システムに追加 (`images.add`)。
    // そのテクスチャを基本色（base_color_texture）として持つ `StandardMaterial` をアセットに追加し、
    // ハンドル（参照ID: debug_material）を取得します。
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // --- 2. 3D基本形状（Primitives）のメッシュ生成 ---
    // 数学的形状（Cuboid, Sphereなど）はそのままでは描画できません。
    // `meshes.add(...)` に渡すことで、Rustの `From` トレイトにより自動的に `Mesh` へ変換されアセット化されます。
    let shapes = [
        meshes.add(Cuboid::default()),        // 直方体
        meshes.add(Tetrahedron::default()),   // 四面体
        meshes.add(Capsule3d::default()),     // 3Dカプセル
        meshes.add(Torus::default()),         // トーラス（ドーナツ型）
        meshes.add(Cylinder::default()),      // 円柱
        meshes.add(Cone::default()),          // 円錐
        meshes.add(ConicalFrustum::default()),// 円錐台
        // 球体（Sphere）を高精度なICOメッシュ（20面体ベース）に分割して生成
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        // 球体をUV格子分割（経線32/緯線18）で生成
        meshes.add(Sphere::default().mesh().uv(32, 18)),
        meshes.add(Segment3d::default()),     // 3D線分
        // 複数頂点を結ぶ折れ線（Polyline3d）の生成
        meshes.add(Polyline3d::new(vec![
            Vec3::new(-0.5, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
        ])),
    ];

    // --- 3. 2D形状の押し出し（Extrusion）メッシュ生成 ---
    // 2D平面図形（Rectangle, Circleなど）に厚み（奥行き=1.0）を与えて3D化します。
    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),      // 長方形を押し出し -> 角柱
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),      // 2Dカプセルを押し出し
        meshes.add(Extrusion::new(Annulus::default(), 1.)),        // 二重円（アナラス）を押し出し
        meshes.add(Extrusion::new(Circle::default(), 1.)),         // 円を押し出し -> 円柱
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),        // 楕円を押し出し -> 楕円柱
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)), // 正多角形を押し出し
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),     // 三角形を押し出し -> 三角柱
        // 5つの頂点を持つ凸多角形（ConvexPolygon）を作成して押し出し
        meshes.add(Extrusion::new(
            ConvexPolygon::new(vec![
                Vec2::new(0.0, 0.8),
                Vec2::new(-0.47, 0.25),
                Vec2::new(-0.47, -0.65),
                Vec2::new(0.47, -0.65),
                Vec2::new(0.47, 0.25),
            ])
            .unwrap(),
            1.0,
        )),
    ];

    // --- 4. リング（中空）形状の押し出しメッシュ生成 ---
    // `.to_ring(THICKNESS)` や `Ring::new(...)` で内側をくり抜いた枠線状の2D図形を作り、それを3D押し出しします。
    let ring_extrusions = [
        meshes.add(Extrusion::new(Rectangle::default().to_ring(THICKNESS), 1.)), // 中空長方形（フレーム状）
        meshes.add(Extrusion::new(Capsule2d::default().to_ring(THICKNESS), 1.)), // 中空2Dカプセル
        // 外側の円（半径1.0）と内側の円（半径0.5）からリングを生成して押し出し
        meshes.add(Extrusion::new(
            Ring::new(Circle::new(1.0), Circle::new(0.5)),
            1.,
        )),
        meshes.add(Extrusion::new(Circle::default().to_ring(THICKNESS), 1.)),     // 中空円（パイプ状）
        // 楕円のリング（内外の楕円を手動計算してRing構造を作成）
        meshes.add(Extrusion::new(
            {
                let outer = Ellipse::default();
                let mut inner = outer;
                inner.half_size -= Vec2::splat(THICKNESS);
                Ring::new(outer, inner)
            },
            1.,
        )),
        meshes.add(Extrusion::new(
            RegularPolygon::default().to_ring(THICKNESS),
            1.,
        )),
        meshes.add(Extrusion::new(Triangle2d::default().to_ring(THICKNESS), 1.)),
    ];

    // --- 5. 前列（Front Row）への基本形状オブジェクト配置 ---
    let num_shapes = shapes.len();
    for (i, shape) in shapes.into_iter().enumerate() {
        // commands.spawn(...) で新しいEntityを生成し、以下のコンポーネント群をタプルで付与します
        commands.spawn((
            Mesh3d(shape), // 描画するメッシュハンドル（Newtypeラップ）
            MeshMaterial3d(debug_material.clone()), // 適用するマテリアルハンドル（参照カウントを増やすため .clone()）
            Transform::from_xyz(
                // X座標：等間隔に並べる計算
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,           // Y座標：高さ 2.0
                Row::Front.z(),// Z座標：前列のZ位置
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)), // X軸まわりに -45度 (-π/4) だけ傾ける
            Shape,     // マーカーコンポーネント（回転用Queryで対象を判別するため）
            Row::Front,// 独自の列管理コンポーネント（現在の位置状態を記録）
        ));
    }

    // --- 6. 中列（Middle Row）への押し出し形状オブジェクト配置 ---
    let num_extrusions = extrusions.len();
    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                Row::Middle.z(),
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
            Row::Middle,
        ));
    }

    // --- 7. 後列（Rear Row）へのリング押し出し形状オブジェクト配置 ---
    let num_ring_extrusions = ring_extrusions.len();
    for (i, shape) in ring_extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_ring_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                Row::Rear.z(),
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
            Row::Rear,
        ));
    }

    // --- 8. 点光源（PointLight）の配置 ---
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true, // 影の描画（シャドウマップ）を有効化
            intensity: 10_000_000.,    // 光の強さ（ルーメン相当）
            range: 100.0,              // 光が届く最大距離
            shadow_depth_bias: 0.2,    // 影の自己干渉（シャドウアクネ）を防ぐバイアス値
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0), // 光源の配置位置 (X:8, Y:16, Z:8)
    ));

    // --- 9. 床面（Ground Plane）の配置 ---
    commands.spawn((
        // 50x50 単位の平面メッシュを生成して配置
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        // シルバー色の単色マテリアルをアタッチ
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    // --- 10. 3Dカメラ（Camera3d）の配置 ---
    commands.spawn((
        Camera3d::default(),
        // (0, 7, 14) の位置から原点付近 (0, 1, 0) を見下ろすように設定（上方向は Y軸）
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    // --- 11. UIテキスト（操作案内）の配置 ---
    let mut text = "\
        Press 'R' to pause/resume rotation\n\
        Press 'Tab' to cycle through rows"
        .to_string();
    #[cfg(not(target_arch = "wasm32"))]
    text.push_str("\nPress 'Space' to toggle wireframes");

    commands.spawn((
        Text::new(text), // 画面に表示するテキストコンポーネント
        Node {
            position_type: PositionType::Absolute, // 絶対座標指定（画面上の固定位置）
            top: px(12),  // 画面上部から 12 ピクセル
            left: px(12), // 画面左部から 12 ピクセル
            ..default()
        },
    ));
}

// ============================================================================
// 4. rotate システム：毎フレームの回転アニメーション
// ============================================================================
/// 画面上のオブジェクトをY軸まわりに継続回転させるシステム
fn rotate(
    // Query（クエリ）: 「Shape コンポーネントを持つ Entity の Transform の可変参照 (&mut Transform)」を検索取得
    mut query: Query<&mut Transform, With<Shape>>,
    time: Res<Time>, // 時間リソース（前フレームからの経過時間 delta_secs を取得するため）
) {
    // 該当するすべてのオブジェクトに対してループ処理
    for mut transform in &mut query {
        // time.delta_secs() を使うことで、FPSが変動しても一定の速度（0.5ラジアン/秒）で回転します
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

// ============================================================================
// 5. uv_debug_texture 関数：動的テクスチャ画像の生成
// ============================================================================
/// 8x8ピクセルのパレット回転パターンによるデバッグ用RGBAテクスチャ画像を生成
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    // 8つのRGBAカラー（1カラーあたり4バイト = 計32バイト）のカラーパレット配列
    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255,
        102, 255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    // 8x8 ピクセル x 4個のチャンネル(RGBA) = 256 バイトの画像データバッファを用意
    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];

    // 各行（y = 0..8）に対してパレットをコピーし、カラーをずらしていく
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        // スライスにパレットの32バイトデータをコピー
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        // パレット配列の要素を右に4バイト（1色分）循環シフトさせ、格子状のチェッカー模様を作成
        palette.rotate_right(4);
    }

    // 生のバイト配列から Bevy が理解できる 2D `Image` 構造体を組み立てて返却
    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb, // 標準的な 8bit sRGB RGBA フォーマット
        RenderAssetUsages::RENDER_WORLD, // GPUレンダリングワールドでのみ使用する設定
    )
}

// ============================================================================
// 6. toggle_wireframe システム：ワイヤーフレーム表示のトグル切り替え
// ============================================================================
#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>, // ワイヤーフレーム全体設定のリソース可変参照
    keyboard: Res<ButtonInput<KeyCode>>,          // キーボード入力の状態を保持するリソース
) {
    // スペースキーが押された瞬間（just_pressed）かを判定
    if keyboard.just_pressed(KeyCode::Space) {
        // ワイヤーフレームのグローバル有効フラグを反転 (true ↔ false)
        wireframe_config.global = !wireframe_config.global;
    }
}

// ============================================================================
// 7. Row 列挙型 & advance_rows システム：列（奥行き）管理と移動
// ============================================================================

/// オブジェクトが配置されている列の位置状態を表すカスタムコンポーネント
#[derive(Component, Clone, Copy)]
enum Row {
    Front,  // 前列
    Middle, // 中列
    Rear,   // 後列
}

impl Row {
    /// 各列に対応する Z座標 の値を返すヘルパーメソッド
    fn z(self) -> f32 {
        match self {
            Row::Front => Z_EXTENT / 2.,   // 前列: +4.0
            Row::Middle => 0.,             // 中列:  0.0
            Row::Rear => -Z_EXTENT / 2.,  // 後列: -4.0
        }
    }

    /// 列の状態を前・中・後で巡回（Front -> Rear -> Middle -> Front）させる状態遷移メソッド
    fn advance(self) -> Self {
        match self {
            Row::Front => Row::Rear,
            Row::Middle => Row::Front,
            Row::Rear => Row::Middle,
        }
    }
}

/// Tabキー押下時に全オブジェクトのRow（列）状態を進め、Z座標を更新するシステム
fn advance_rows(
    // Row コンポーネントと Transform コンポーネントの可変参照をペアで取得
    mut shapes: Query<(&mut Row, &mut Transform), With<Shape>>,
) {
    for (mut row, mut transform) in &mut shapes {
        // 1. 列の状態を次の位置へと更新 (*row デ参照して値を書き換え)
        *row = row.advance();
        // 2. 更新された列に対応する Z座標を Transform に反映
        transform.translation.z = row.z();
    }
}
