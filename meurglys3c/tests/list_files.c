#include "meu3.h"
#include "stdio.h"

int main(void) {
    MEU3_Error err = -1;
    MEU3_PACKAGE* pack = meu3_package_dir("test_dir", &err);
    if(!pack) {
        return 1;
    }
    struct MEU3_FileList flist = meu3_package_get_file_list(pack, &err);
    if(flist.len != 2)
        return 1;
    for(size_t i = 0; i < flist.len; i++){
        printf("%s\n", flist.data[i]);
    }
    meu3_free_file_list(flist);
    meu3_free_package(pack);
    return 0;
}
