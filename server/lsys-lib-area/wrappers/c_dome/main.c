// 需要先运行 cargo build --features "lib-clib" 后编译
#include <stdio.h>
#include "lsys_lib_area.h"
void test_code_related(CAreaDao* area_dao){
    char * err=NULL;
    printf("code related \n");
    CAreaRelatedItemVecs *area_vecr;
    if (lib_area_code_related("4414",area_dao,&area_vecr,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    long lenr1 = area_vecr->len;
    CAreaRelatedItemVec *tmpr1 = area_vecr->data;
    while (lenr1-- > 0) {
        long len2 = tmpr1->len;
        CAreaRelatedItem *tmp2 = tmpr1->data;
        while (len2-- > 0) {
            printf("%s:%s\n",tmp2->name,tmp2->selected?"[*]":"[ ]");
            tmp2++;
        }
        tmpr1++;
    }
    lib_area_release_related_vecs(area_vecr);

    CAreaRelatedItemVecs *tmpt;
    if (lib_area_code_related("121212121212",area_dao,&tmpt,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    lib_area_release_related_vecs(tmpt);

}
int test_reload(CAreaDao** area_dao){
    char * err=NULL;

    printf("code reload \n");
    int ret2=lib_area_code_reload(area_dao,&err);
    if (ret2!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ret2;
    }

    printf("geo reload \n");
    int ret3=lib_area_geo_reload(area_dao,&err);
    if (ret3!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ret2;
    }
    return 0;
}

void test_code_find(CAreaDao* area_dao){
    char * err=NULL;
    printf("code find \n");
    CAreaItemVec* area_vec;
    if (lib_area_code_find("441403131",area_dao,&area_vec,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    long len=area_vec->len;
    CAreaItem * tmp=area_vec->data;
    while (len-->0){
        printf("%s [%s]\n",tmp->name,tmp->code);
        tmp++;
    }
    lib_area_release_item_vec(area_vec);




    CAreaItemVec* tmp1;
    if (lib_area_code_find("121212121212",area_dao,&tmp1,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    lib_area_release_item_vec(tmp1);
}


void test_code_childs(CAreaDao* area_dao){
    char * err=NULL;
    printf("code childs \n");
    CAreaItemVec* area_vec2;
    if (lib_area_code_childs("",area_dao,&area_vec2,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    long len2=area_vec2->len;
    CAreaItem * tmp2=area_vec2->data;
    while (len2-->0){
        printf("%s [%s]\n",tmp2->name,tmp2->code);
        tmp2++;
    }
    lib_area_release_item_vec(area_vec2);


    CAreaItemVec* tmp;
    if (lib_area_code_childs("12213123123123",area_dao,&tmp,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    lib_area_release_item_vec(tmp);
}

void test_geo_search(CAreaDao* area_dao){
    char * err=NULL;
    printf("geo search \n");
    CAreaItemVec* area_vecg;
    if (lib_area_geo_search(22.57729, 113.89409,area_dao,&area_vecg,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    long leng=area_vecg->len;
    CAreaItem * tmpg=area_vecg->data;
    while (leng-->0){
        printf("%s [%s]\n",tmpg->name,tmpg->code);
        tmpg++;
    }
    lib_area_release_item_vec(area_vecg);

    CAreaItemVec* tmp;
    if (lib_area_geo_search(0.0, 0.0,area_dao,&tmp,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    lib_area_release_item_vec(tmp);
}

void test_code_search(CAreaDao* area_dao){
    char * err=NULL;
    printf("code search \n");
    CAreaItemVecs* area_vec1;
    if (lib_area_code_search("guangdong",10,area_dao,&area_vec1,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    long len1 = area_vec1->len;
    CAreaItemVec *tmp1 = area_vec1->data;
    while (len1-- > 0) {
        long len2 = tmp1->len;
        CAreaItem *tmp2 = tmp1->data;
        printf("address:");
        while (len2-- > 0) {
            printf("%s ",tmp2->name);
            tmp2++;
        }
        printf("\n");
        tmp1++;
    }
    lib_area_release_item_vecs(area_vec1);



    CAreaItemVecs* tmp;
    if (lib_area_code_search("fadsfasdadfasd",10,area_dao,&tmp,&err)!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return ;
    }
    lib_area_release_item_vecs(tmp);


}




int main() {
    CAreaDao* area_dao;
    char * err;
    char gz=1;
    int ret=lib_area_init_csv(
            "../../data/2023-7-area-code.csv.gz",
            "../../data/2023-7-area-geo.csv.gz",
            0,
            &gz,
            &area_dao,
            &err
        );
    if (ret!=0){
        printf("err:%s\n",err);
        lib_area_release_error_str(err);
        return 0;
    }
    test_code_search(area_dao);
    test_code_childs(area_dao);
    test_code_find(area_dao);
    test_code_related(area_dao);
    test_geo_search(area_dao);
    if(test_reload(&area_dao)!=0){
        return 0;
    }
    test_code_search(area_dao);
    test_code_childs(area_dao);
    test_code_find(area_dao);
    test_code_related(area_dao);
    test_geo_search(area_dao);
    lib_area_release_area_dao(area_dao);
    return 0;
}
