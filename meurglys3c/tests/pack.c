#include "meu3.h"

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    meu3_free_package(pack);
    return 0;
}
