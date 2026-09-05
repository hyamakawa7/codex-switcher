# fork版Codex Switcherの更新手順

このPCでは、公式の変更をforkのソースへ取り込み、Linux向けの自動起動機能を維持して `.deb` を再ビルド・インストールする。

## 現在の構成

2026-09-06にローカルで確認した内容。将来作業するときは `git remote -v` と `git status` で再確認する。

| 対象 | 現在の値・状態 |
|---|---|
| 作業ディレクトリ | `/home/mt/Documents/playground/codex-switcher` |
| 自分のfork（origin） | `https://github.com/hyamakawa7/codex-switcher.git` |
| 公式リポジトリ | `https://github.com/Lampese/codex-switcher` |
| 公式用リモート（upstream） | 未登録。後述の手順で追加する |
| 今回のブランチ | `feature/linux-autostart-tray` |
| 今回の変更 | 未コミット。GitHubへの保存はまだ完了していない |
| インストール版 | `0.2.12`を元にしたローカルビルド |
| 自動起動の設定 | Launch at login / Start hidden in tray ともにON |
| 再ログイン | ユーザーによる動作確認済み |

この文書の作成ではリモート追加、commit、push、merge、更新先の変更は実行していない。

## アプリ内のUpdateボタンについて

`src-tauri/tauri.conf.json` の更新先は、現在も公式の `Lampese/codex-switcher/releases/latest/download/latest.json`。`src/components/UpdateChecker.tsx` は起動時に更新を確認し、利用者が **Update** を押すと配布物のダウンロード・インストールを開始する。通知だけでインストールする実装ではない。

forkをcloneしても、この更新先は自動では自分のforkへ変わらない。公式版への更新が成功すると、今回追加した機能を含まないバイナリに置き換わる可能性がある。今回の機能を維持する間は **Later** を選び、下記のソース取り込み手順を使う。公式 `.deb` の直接インストールでも同じ点に注意する。

ビルド時の `createUpdaterArtifacts=false` は署名付き更新成果物の生成を止める指定であり、アプリ内の更新確認を無効化する設定ではない。今回のビルドには `__TAURI_BUNDLE_TYPE` に関する警告もあったため、アプリ内更新が必ず成功する／必ず失敗するとは扱わない。

自分のforkからアプリ内更新を配信したい場合は、別の機能として更新URL、署名鍵と公開鍵、CIの秘密情報、配布成果物、バージョン運用を整備する。URLの置換だけでは完了しない。

## 初回だけ：今回の変更をforkへ保存する

現在の未コミット変更を保存してから公式の変更を取り込む。以下は今後実行する手順。

```bash
cd /home/mt/Documents/playground/codex-switcher
git status --short --branch
git diff --stat
git diff --check
```

未追跡の `docs/`、対応表、新規Rust・Reactファイルも内容を確認する。機能の関連ファイルを明示してステージする。

```bash
git add README.md docs/ referent-table-linux-autostart.md referent-table-fork-updates.md src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/auth/storage.rs src-tauri/src/commands/window.rs src-tauri/src/lib.rs src-tauri/src/tray.rs src-tauri/src/types.rs src-tauri/src/startup.rs src/App.tsx src/components/StartupSettings.tsx
git diff --cached --stat
git diff --cached --check
git commit -m "feat: add Linux login and hidden startup preferences"
git push -u origin feature/linux-autostart-tray
```

自分のfork内で、このブランチから `main` へPRを作り、レビューして取り込む。PRの取り込み先が **hyamakawa7/codex-switcher の main** であることを確認する。公式リポジトリへ提案するPRは別の操作。

取り込み後、ローカルのmainを更新する。

```bash
git switch main
git pull --ff-only origin main
```

この後の手順は、今回の機能がforkのmainへ保存済みであることを前提とする。

## 初回だけ：公式用のリモートを登録する

```bash
git remote -v
```

`upstream` がない場合にだけ実行する。

```bash
git remote add upstream https://github.com/Lampese/codex-switcher.git
git remote -v
```

既にある場合はURLが公式を指しているか確認する。`origin` は自分のforkのまま維持する。この構成は [GitHubのfork用リモート設定手順](https://docs.github.com/en/pull-requests/how-tos/work-with-forks/configuring-a-remote-repository-for-a-fork) に対応する。

## 更新のたびに：公式のリリースを作業ブランチへ取り込む

1. 作業ツリーが空であることを確認する。未保存の変更があれば先に対応する。
2. forkのmainを取得し、公式の変更を取得する。
3. [公式リリース一覧](https://github.com/Lampese/codex-switcher/releases)から取り込むリリースを選ぶ。開発途中のmainではなく、確認したリリースのコミットを使う運用を推奨する。

```bash
git status --short
git switch main
git pull --ff-only origin main
git fetch upstream --prune
```

以下の `vX.Y.Z` は実在するリリースタグ、`x-y-z` はそのバージョンをケバブケースで記載する置換箇所。公式タグを専用の参照へ取得するので、自分のforkの同名タグと混同しない。

```bash
git fetch upstream refs/tags/vX.Y.Z:refs/remotes/upstream/releases/vX.Y.Z
git switch -c feature/update-upstream-x-y-z
git log --oneline HEAD..refs/remotes/upstream/releases/vX.Y.Z
git merge --no-ff --no-commit refs/remotes/upstream/releases/vX.Y.Z
```

merge後に差分を確認し、競合があれば公式の変更と今回の動作の両方を残すよう解決する。特に次のファイルを確認する。

| ファイル | 維持・確認する内容 |
|---|---|
| `src-tauri/src/lib.rs` | ウィンドウ生成前の表示設定、Single Instanceの最初の登録、コマンド登録 |
| `src-tauri/src/startup.rs` | XDG登録・無効化、設定取得、--autostart付き二重起動の判定 |
| `src-tauri/src/types.rs`、`auth/storage.rs` | 旧設定でstart_hidden=false、読み書きエラー、途中失敗で元ファイルを壊さない保存 |
| `src-tauri/src/tray.rs`、`commands/window.rs` | 非表示時のトレイ表示、復帰・終了 |
| `src/App.tsx`、`src/components/StartupSettings.tsx` | Linux限定の2項目、保存中の無効化、失敗時の表示 |
| `src-tauri/Cargo.toml`、`Cargo.lock` | Single Instance依存関係と公式側の依存更新 |
| `src-tauri/tauri.conf.json` | 更新URL・公開鍵・ウィンドウ設定に変更がないか |

競合中は `git status` で対象を確認し、解決したファイルだけ `git add` する。中止する場合は `git merge --abort`。未解決のままビルド・導入しない。`reset --hard upstream/main` やfork同期の強制上書きは、この運用には使わない。

公式が同等の自動起動機能を追加した場合は、設定保存先・既定値・二重起動・非表示動作を比較してから、重複実装を残すか公式へ移行するか決める。通常のmergeは公式とforkの履歴を統合する操作。[GitHubの同期手順](https://docs.github.com/en/pull-requests/how-tos/work-with-forks/syncing-a-fork) も参照。

## 取り込んだソースを検証して保存する

```bash
pnpm install --frozen-lockfile
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
git diff --cached --check
git status
```

依存関係の競合解決でlockfileを更新する必要がある場合は、まず競合内容を確認し、対象ツールで再生成して差分をレビューする。`--frozen-lockfile` の失敗を理由にlockfileを無条件で削除しない。今回のRustテストは52件だったが、今後は件数固定ではなく必要なテストの成功を確認する。

確認した取り込み内容をコミットする。取り込み対象が既に含まれていて変更がなければコミットは不要。

```bash
git commit -m "Merge upstream release vX.Y.Z"
git push -u origin feature/update-upstream-x-y-z
```

`pnpm release` はバージョン更新・commit・tag作成を行う配布用スクリプトなので、単なるローカルdebの更新には使わない。独自のバージョン番号が必要になった場合は別途ルールを決める。同じパッケージバージョンでも独自の変更があり得るため、ビルドとGitコミットの対応を保存する。

## インストール前に直前の稼働版を保存する

トレイの **Quit** で終了してから保存する。毎回新しいディレクトリを使う。

```bash
backup_dir="$HOME/.local/state/codex-switcher/backups/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$backup_dir"
chmod 700 "$backup_dir"
cp -a /usr/bin/codex-switcher "$backup_dir/codex-switcher"
cp -a "$HOME/.codex-switcher/settings.json" "$backup_dir/settings.json"
autostart_file="${XDG_CONFIG_HOME:-$HOME/.config}/autostart/codex-switcher.desktop"
cp -a "$autostart_file" "$backup_dir/autostart.desktop"
printf '%s\n' "$autostart_file" > "$backup_dir/autostart-path.txt"
dpkg-query -W codex-switcher > "$backup_dir/package-version.txt"
```

現在のこのPCでは両設定ファイルが存在する。存在しない環境で使う場合は「元は存在しなかった」と記録し、復元時に新規ファイルだけを除去するよう手順を調整する。

直前にインストールした `.deb` も `previous.deb` としてバックアップする。現在の0.2.12なら、ビルドで上書きする前に次を実行できる。

```bash
cp 'src-tauri/target/release/bundle/deb/Codex Switcher_0.2.12_amd64.deb' "$backup_dir/previous.deb"
```

以後は次節で保管した稼働版のdebを使う。ビルドディレクトリにあるファイルが稼働版と一致する保証はないため、バージョン番号だけで選ばず保存したコミット・ハッシュも確認する。

設定形式が変わる更新では、必要に応じてアカウントデータもアプリの暗号化エクスポート機能で保存する。認証情報を含むバックアップをGitへ追加しない。

## debをビルドしてインストールする

```bash
pnpm tauri build --bundles deb --config '{"bundle":{"createUpdaterArtifacts":false}}'
```

成功時に表示された **今回生成されたamd64.debのパス** を指定する。末尾の `.deb` まで含める。下記パスのバージョンは例なので、ビルド結果に合わせる。

```bash
deb_path='/home/mt/Documents/playground/codex-switcher/src-tauri/target/release/bundle/deb/Codex Switcher_0.2.12_amd64.deb'
test -f "$deb_path"
dpkg-deb --info "$deb_path"
```

各コマンドの成功を確認して次へ進む。成果物をビルドディレクトリ外へ保管する。

```bash
build_dir="$HOME/.local/state/codex-switcher/builds/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$build_dir"
cp "$deb_path" "$build_dir/codex-switcher.deb"
git rev-parse HEAD > "$build_dir/source-commit.txt"
sha256sum "$build_dir/codex-switcher.deb" > "$build_dir/SHA256SUMS"
sudo dpkg -i "$build_dir/codex-switcher.deb"
/usr/bin/codex-switcher
```

sudo認証は端末で行う。インストールに失敗した場合はエラーを確認してから復旧する。保存ディレクトリとGitコミットを更新作業の記録に残す。

## インストール後の確認

| 確認 | 期待結果 |
|---|---|
| 設定2項目 | Linuxデスクトップの右上メニューに表示される |
| OFF/OFF | 自動起動は無効、手動起動で画面表示 |
| OFF/ON | 自動起動は無効、手動起動で非表示 |
| ON/OFF | 自動起動ファイルから起動すると画面表示 |
| ON/ON | 自動起動ファイルから起動するとちらつかずトレイ常駐 |
| トレイ操作 | Open Codex Switcherで復帰、Quitで終了 |
| 常駐中の手動起動 | 同じプロセスの画面が開く |
| 常駐中の--autostart | 画面状態を変えず、常駐プロセスも増えない |
| 最終設定 | 両項目ONに戻す |
| 次回ログイン | 自動でトレイ常駐し、メイン画面が出ない |

```bash
desktop-file-validate "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/codex-switcher.desktop"
pgrep -ax codex-switcher
```

自動起動ファイルの直接実行は、アプリ終了後に通常のデスクトップ端末から次を実行する。

```bash
gio launch "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/codex-switcher.desktop"
```

初回の移行・再ログイン確認は済んでいる。更新のたびに旧外部ヘルパーへ戻す必要はない。更新後も `/usr/bin/codex-switcher` を同じ場所へ入れ替える限り、自動起動ファイルは同じ実行パスを利用できる。

検証後、更新ブランチのPRを自分のforkのmainへ取り込む。実際にインストールしたビルドのコミットは、上で保存した `source-commit.txt` を基準に追跡する。

## 更新に問題があったら直前の稼働版へ戻す

トレイのQuitで終了し、`backup_dir` を今回の更新前に作成したバックアップ先へ設定する。アプリが応答しない場合は対象プロセスを確認して終了する。

```bash
sudo dpkg -i "$backup_dir/previous.deb"
cp -a "$backup_dir/settings.json" "$HOME/.codex-switcher/settings.json"
autostart_file=$(cat "$backup_dir/autostart-path.txt")
cp -a "$backup_dir/autostart.desktop" "$autostart_file"
/usr/bin/codex-switcher
```

旧debが保存されていない場合は、バックアップしたバイナリを `sudo install -o root -g root -m 755 "$backup_dir/codex-switcher" /usr/bin/codex-switcher` で復元できるが、パッケージ管理上のバージョンや他の同梱ファイルは戻らない。通常は旧debから戻す。

ここで使うのは **直前の稼働版** のバックアップ。初回移行前の `20260906-005018` は今回の自動起動機能を持たない版と旧ヘルパーのバックアップなので、通常の更新失敗時には使わない。初回移行そのものを取り消す場合だけ [READMEの復元手順](../README.md#restore-the-previous-local-installation) を使う。

ソースの更新ブランチは原因調査のため残せる。バイナリの復元とGit履歴の変更は別の操作であり、復元のためにmainを強制的に書き換える必要はない。

作成前の対応表: [referent-table-fork-updates.md](../referent-table-fork-updates.md)
