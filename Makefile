# 默认目标
all: @build

# 构建项目
build:
	@cargo build


build-async:
	@cargo build --features "async"

# 以 release 模式构建项目
release:
	@cargo build --release

# 运行项目
run:
	@cargo run

# 使用 `async` 特性运行项目
run-async-server:
	@cargo run --features "async" launch-x-server --port 8080

# 清理项目，移除 target 目录和 Cargo.lock 文件
clean:
	@cargo clean

# 检查代码，不进行编译
check:
	@cargo check

# 更新依赖
update:
	@cargo update



# 安装musl release版本
build-musl:
	@cargo build --target x86_64-unknown-linux-musl --release

run-musl-async-server:
	@cargo build --target x86_64-unknown-linux-musl --release
	@./target/x86_64-unknown-linux-musl/release/luckyapi launch-x-server --port 8080

# 伪目标 忽略这些名字的文件
.PHONY: all build release run run-async clean check update

# 使用hyperfine测试
hyperfine:
	@hyperfine --warmup 3 --cleanup 'rm -rf *.zip'  -L args '"[{\"filepath\": \"pictures\"}]"'  --runs '10' './target/release/luckyapi zip -f {args}'