#include "meu3.h"

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    bool res = meu3_write_package("dump/pack", pack, &err);
    if (!res || err != NoError)
        return 1;
    MEU3_PACKAGE* pack2 = meu3_load_package("dump/pack.m3pkg", &err);
    bool has = meu3_package_has(pack2, "text.txt", &err);
    if(!has || err != NoError)
        return 1;
    has = meu3_package_has(pack2, "nested/index.html", &err);
    if(!has || err != NoError)
        return 1;
    meu3_free_package(pack);
    meu3_free_package(pack2);
    return 0;
}
