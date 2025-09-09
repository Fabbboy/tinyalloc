work_dir := `pwd`
build_dir := work_dir + "/build"
tests_dir := build_dir + "/tests"

cmake := env("CMAKE", "cmake")
ctest := env("CTEST", "ctest")
rm := "rm -rf"

cmake_flags := "Ninja"

default: build

build:
    {{cmake}} -B {{build_dir}} -G{{cmake_flags}}
    {{cmake}} --build {{build_dir}}

clean:
    {{rm}} {{build_dir}}

test: build
    {{ctest}} --test-dir {{tests_dir}} --output-on-failure

rebuild: clean build
