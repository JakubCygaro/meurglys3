#include "meu3.h"
#include <stddef.h>
#include <stdio.h>

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    (void)meu3_package_get_compression(pack, &err);
    if (err != NoError)
        return 1;
    (void)meu3_package_get_version(pack, &err);
    if (err != NoError)
        return 1;
    size_t len = 0;
    MEU3_BYTES data = meu3_package_get_data_ptr(pack, "text.txt", &len, &err);
    if (err != NoError || len == 0)
        return 1;
    meu3_free_package(pack);
    return 0;
}
