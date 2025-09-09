#define NOB_IMPLEMENTATION
#define NOB_STRIP_PREFIX
#include "nob.h"

bool build(void) {
    Cmd cmd = {0};
    cmd_append(&cmd, "cmake", "-B", "build", "-GNinja");
    if (!cmd_run(&cmd)) return false;
    
    cmd.count = 0;
    cmd_append(&cmd, "cmake", "--build", "build");
    bool result = cmd_run(&cmd);
    cmd_free(cmd);
    return result;
}

bool clean(void) {
    Cmd cmd = {0};
    cmd_append(&cmd, "rm", "-rf", "build");
    bool result = cmd_run(&cmd);
    cmd_free(cmd);
    return result;
}

bool test(void) {
    if (!build()) return false;
    
    Cmd cmd = {0};
    cmd_append(&cmd, "ctest", "--test-dir", "build/tests", "--output-on-failure");
    bool result = cmd_run(&cmd);
    cmd_free(cmd);
    return result;
}

bool rebuild(void) {
    return clean() && build();
}

int main(int argc, char **argv) {
    NOB_GO_REBUILD_URSELF(argc, argv);
    
    const char *command = argc > 1 ? argv[1] : "build";
    
    if (strcmp(command, "build") == 0) {
        return build() ? 0 : 1;
    } else if (strcmp(command, "clean") == 0) {
        return clean() ? 0 : 1;
    } else if (strcmp(command, "test") == 0) {
        return test() ? 0 : 1;
    } else if (strcmp(command, "rebuild") == 0) {
        return rebuild() ? 0 : 1;
    }
    
    return 1;
}