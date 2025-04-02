#include "meu3.h"

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    bool has = meu3_package_has(pack, "text.txt", &err);
    if(!has || err != NoError) 
        return 1;
    has = meu3_package_has(pack, "nested/index.html", &err);
    if(!has || err != NoError) 
        return 1;
    meu3_free_package(pack);
    return 0;
}
