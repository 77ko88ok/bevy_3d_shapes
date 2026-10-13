## coding tips
### text
- bevyでテキストを表示にするには、Stringデータを保持するのではなく、Bevy::textにわたす
- しかしまだ表示できてない

### wsl GPU renderer [^1]
~~~rust
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(
                    WgpuSettings {
                        instance_flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                        ..default()
                    },
                )),
                ..default()
            }),
~~~



[^1]: https://github.com/bevyengine/bevy/discussions/22851