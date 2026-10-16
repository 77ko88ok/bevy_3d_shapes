//! Here we use shape primitives to generate meshes for 3d objects as well as attaching a runtime-generated patterned texture to each 3d object.
//!
//! "Shape primitives" here are just the mathematical definition of certain shapes, they're not meshes on their own! A sphere with radius `1.0` can be defined with [`Sphere::new(1.0)`][Sphere::new] but all this does is store the radius. So we need to turn these descriptions of shapes into meshes.
//!
//! While a shape is not a mesh, turning it into one in Bevy is easy. In this example we call [`meshes.add(/* Shape here! */)`][`Assets<A>::add`] on the shape, which works because the [`Assets<A>::add`] method takes anything that can be turned into the asset type it stores. There's an implementation for [`From`] on shape primitives into [`Mesh`], so that will get called internally by [`Assets<A>::add`].
//!
//! [`Extrusion`] lets us turn 2D shape primitives into versions of those shapes that have volume by extruding them. A 1x1 square that gets wrapped in this with an extrusion depth of 2 will give us a rectangular prism of size 1x1x2, but here we're just extruding these 2d shapes by depth 1.
//!
//! The material applied to these shapes is a texture that we generate at run time by looping through a "palette" of RGBA values (stored adjacent to each other in the array) and writing values to positions in another array that represents the buffer for an 8x8 texture. This texture is then registered with the assets system just one time, with that [`Handle<StandardMaterial>`] then applied to all the shapes in this example.
//!
//! The mesh and material are [`Handle<Mesh>`] and [`Handle<StandardMaterial>`] at the moment, neither of which implement `Component` on their own. Handles are put behind "newtypes" to prevent ambiguity, as some entities might want to have handles to meshes (or images, or materials etc.) for different purposes! All we need to do to make them rendering-relevant components is wrap the mesh handle and the material handle in [`Mesh3d`] and [`MeshMaterial3d`] respectively.
//!
//! You can toggle wireframes with the space bar except on wasm. Wasm does not support
//! `POLYGON_MODE_LINE` on the gpu.

use std::f32::consts::PI;

#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    asset::RenderAssetUsages,
    color::palettes::basic::SILVER,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
    render::{render_resource::{Extent3d, TextureDimension, TextureFormat}, RenderPlugin, settings::{WgpuSettings, RenderCreation, InstanceFlags}},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(
                    WgpuSettings {
                        instance_flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                        ..default()
                    },
                )),
                ..default()
            }),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rotate.run_if(input_toggle_active(true, KeyCode::KeyR)),
                advance_rows.run_if(input_just_pressed(KeyCode::Tab)),
                #[cfg(not(target_arch = "wasm32"))]
                toggle_wireframe,
            ),
        )
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 14.0;
const Z_EXTENT: f32 = 8.0;
const THICKNESS: f32 = 0.1;

fn setup(
    mut commands: Commands,
    // Entity生成用コマンドキュー
    // spawn() を呼ぶと Entity が生成予約される

    mut meshes: ResMut<Assets<Mesh>>,
    // Meshアセット管理リソース
    // GPUへ送る3D形状を登録する場所

    mut images: ResMut<Assets<Image>>,
    // テクスチャ(Image)管理リソース

    mut materials: ResMut<Assets<StandardMaterial>>,
    // PBRマテリアル管理リソース
) {
    //----------------------------------------------------------
    // デバッグテクスチャ付きマテリアルを作成
    //----------------------------------------------------------

    let debug_material = materials.add(
        StandardMaterial {
            // uv_debug_texture()で生成した画像を
            // テクスチャとして使用する

            base_color_texture: Some(
                images.add(
                    uv_debug_texture()
                )
            ),

            // 他のパラメータはデフォルト
            ..default()
        }
    );

    //----------------------------------------------------------
    // Shape Primitive → Mesh化
    //----------------------------------------------------------

    let shapes = [

        //------------------------------------------------------
        // Cuboid
        //------------------------------------------------------

        meshes.add(
            Cuboid::default()
        ),
        // Shape PrimitiveをMeshへ変換し
        // Assetサーバへ登録

        meshes.add(
            Tetrahedron::default()
        ),

        meshes.add(
            Capsule3d::default()
        ),

        meshes.add(
            Torus::default()
        ),

        meshes.add(
            Cylinder::default()
        ),

        meshes.add(
            Cone::default()
        ),

        meshes.add(
            ConicalFrustum::default()
        ),

        //------------------------------------------------------
        // Icosphere
        //------------------------------------------------------

        meshes.add(
            Sphere::default()
                .mesh()
                .ico(5)
                .unwrap()
        ),
        // 球体をIcosphereとして生成
        // 細分化レベル=5

        //------------------------------------------------------
        // UV Sphere
        //------------------------------------------------------

        meshes.add(
            Sphere::default()
                .mesh()
                .uv(32, 18)
        ),
        // 緯度経度方式のSphere

        meshes.add(
            Segment3d::default()
        ),

        meshes.add(
            Polyline3d::new(vec![
                Vec3::new(-0.5, 0.0, 0.0),
                Vec3::new( 0.5, 0.0, 0.0),
                Vec3::new( 0.0, 0.5, 0.0),
            ])
        ),
        // 折れ線を描画するMesh
    ];

    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
        meshes.add(Extrusion::new(Annulus::default(), 1.)),
        meshes.add(Extrusion::new(Circle::default(), 1.)),
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
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

    let ring_extrusions = [
        meshes.add(Extrusion::new(Rectangle::default().to_ring(THICKNESS), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default().to_ring(THICKNESS), 1.)),
        meshes.add(Extrusion::new(
            Ring::new(Circle::new(1.0), Circle::new(0.5)),
            1.,
        )),
        meshes.add(Extrusion::new(Circle::default().to_ring(THICKNESS), 1.)),
        meshes.add(Extrusion::new(
            {
                // This is an approximation; Ellipse does not implement Inset as concentric ellipses do not have parallel curves
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

// shapeの総数
let num_shapes = shapes.len();

// Mesh配列を順番に取り出す
//
// i:
//    0,1,2,3...
//
// shape:
//    CuboidのMesh
//    SphereのMesh
//    TorusのMesh
//    ...
//
for (i, shape) in shapes.into_iter().enumerate() {

    //--------------------------------------------------
    // Entity生成
    //--------------------------------------------------

    commands.spawn((

        //----------------------------------------------
        // Mesh Component
        //----------------------------------------------

        Mesh3d(shape),

        // Entityが描画する3D形状
        //
        // このEntityはどのMeshを表示するか？
        //
        // をRenderシステムへ伝える


        //----------------------------------------------
        // Material Component
        //----------------------------------------------

        MeshMaterial3d(
            debug_material.clone()
        ),

        // 表面材質
        //
        // UVデバッグ用テクスチャを貼る
        //
        // cloneだがMaterial自体は複製されない
        // Handleだけが複製される
        //
        // 全Entityが同じMaterialを共有


        //----------------------------------------------
        // Transform Component
        //----------------------------------------------

        Transform::from_xyz(

            // X座標
            //
            // 全shapeを横方向に均等配置
            //
            // 左端 -------------- 右端
            //
            // -7                 +7
            //
            -SHAPES_X_EXTENT / 2.
                + i as f32
                    / (num_shapes - 1) as f32
                    * SHAPES_X_EXTENT,

            // Y座標
            //
            // 地面より少し上
            //
            2.0,

            // Z座標
            //
            // Front列の位置
            //
            Row::Front.z(),
        )

        //----------------------------------------------
        // Rotation
        //----------------------------------------------

        .with_rotation(
            Quat::from_rotation_x(
                -PI / 4.
            )
        ),

        // 全shapeを
        // X軸方向へ45度傾ける
        //
        // 平面的なShapeでも
        // 奥行きを見やすくする目的


        //----------------------------------------------
        // Marker Component
        //----------------------------------------------

        Shape,

        // 「このEntityはShapeである」
        //
        // rotate()のQuery対象になる
        //
        // Query<&mut Transform, With<Shape>>


        //----------------------------------------------
        // Row Component
        //----------------------------------------------

        Row::Front,

        // Front列所属
        //
        // 後でTabが押されると
        //
        // Front → Rear
        // Rear → Middle
        // Middle → Front
        //
        // に循環する
    ));
}
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

    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    let mut text = "\
        Press 'R' to pause/resume rotation\n\
        Press 'Tab' to cycle through rows"
        .to_string();
    #[cfg(not(target_arch = "wasm32"))]
    text.push_str("\nPress 'Space' to toggle wireframes");

    commands.spawn((
        Text::new(text),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn rotate(

    //--------------------------------------------------
    // Query
    //--------------------------------------------------

    mut query: Query<

        // 取得したいComponent
        &mut Transform,

        // フィルタ条件
        With<Shape>

    >,

    //--------------------------------------------------
    // Resource
    //--------------------------------------------------

    time: Res<Time>,
    // Bevyが管理するTimeリソース
    //
    // 前フレームから何秒経過したか
    // ゲーム開始から何秒経過したか
    //
    // などが取得できる

) {

    //--------------------------------------------------
    // Shapeタグを持つ全Entityを取得
    //--------------------------------------------------

    for mut transform in &mut query {

        //----------------------------------------------
        // Y軸回転
        //----------------------------------------------

        transform.rotate_y(

            // delta_secs()
            //
            // 前フレームから経過した秒数
            //
            // 60FPSなら
            // 約0.016秒
            //
            // FPS依存を避けるため利用

            time.delta_secs() / 2.
        );
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}

#[derive(Component, Clone, Copy)]
enum Row {
    Front,
    Middle,
    Rear,
}

impl Row {
    fn z(self) -> f32 {
        match self {
            Row::Front => Z_EXTENT / 2.,
            Row::Middle => 0.,
            Row::Rear => -Z_EXTENT / 2.,
        }
    }

    fn advance(self) -> Self {
        match self {
            Row::Front => Row::Rear,
            Row::Middle => Row::Front,
            Row::Rear => Row::Middle,
        }
    }
}

fn advance_rows(mut shapes: Query<(&mut Row, &mut Transform), With<Shape>>) {
    for (mut row, mut transform) in &mut shapes {
        *row = row.advance();
        transform.translation.z = row.z();
    }
}