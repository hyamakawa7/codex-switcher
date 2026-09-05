# Linux自動起動の実機確認（2026-09-06）

対応表: [referent-table-linux-autostart.md](../referent-table-linux-autostart.md)

Ubuntu 24 / X11のこのPCで、インストール済み `/usr/bin/codex-switcher` を確認した。
ブランチは `feature/linux-autostart-tray`。インストール済みバイナリと生成したDebianパッケージ内のバイナリのSHA-256はともに `e808189aed9d6bd46ef970e85a93ec85b1912833908c9f1df61dd92f2646b488`。

## 設定と起動時の画面

実際のメニューのチェックボックスをアクセシビリティAPIで操作し、登録ファイルと設定ファイルへの反映を確認した。

| Launch at login | Start hidden in tray | 登録ファイル | 起動結果 |
|---|---|---|---|
| OFF | OFF | Hidden=true | 手動起動で画面表示 |
| OFF | ON | Hidden=true | 手動起動で非表示、トレイActive |
| ON | OFF | インストール済み実行ファイルと--autostart | desktopファイルの直接起動で画面表示 |
| ON | ON | インストール済み実行ファイルと--autostart | desktopファイルの直接起動で非表示、トレイActive |

非表示起動2ケースは、起動前から15秒間X11のMapNotifyを監視し、メイン画面の表示イベントが0件だった。手動再起動による復帰では同じ監視器が1件を記録し、検出できることも確認した。

- トレイの Open Codex Switcher で画面復帰、Quit でプロセス終了。
- 常駐中の手動起動で既存画面が開き、プロセスIDは不変、常駐プロセス数は1。
- 常駐中の --autostart 起動は、非表示・表示それぞれの画面状態を維持。
- 最終状態は両項目ON、メイン画面非表示、トレイActive、プロセス数1。

## バックアップと復元

バックアップ先は `/home/mt/.local/state/codex-switcher/backups/20260906-005018/`。
旧実行ファイル、自動起動ファイル、外部の起動・非表示化補助ファイル3個を保存した。旧settings.jsonは存在しなかったため `settings-was-absent` を記録した。補助ファイルは内容一致を確認したうえで元の配置から削除した。

復元時は [READMEの手順](../README.md#restore-the-previous-local-installation) の前に次を設定する。

```bash
backup_dir="$HOME/.local/state/codex-switcher/backups/20260906-005018"
```

## ビルドと自動検証

- pnpm build: 成功。
- cargo test --manifest-path src-tauri/Cargo.toml: 52件成功、失敗0。
- pnpm tauri build --bundles deb --config '{"bundle":{"createUpdaterArtifacts":false}}': 成功。
- desktop-file-validate: パッケージのランチャーと移行後の自動起動ファイルで成功。
- git diff --check: 成功。

ビルド時にTauriから `__TAURI_BUNDLE_TYPE` が見つからず、アップデーターがパッケージを更新できない可能性があるという警告が出た。今回のローカルインストール・起動は成功しているが、自動アップデートは検証対象に含めていない。

再ログイン時の動作は、その後ユーザーから正常動作の確認を得た。トレイ作成失敗時のフォールバックは実装を確認したが、実機での故障注入は行っていない。
