import ArticleOutlinedIcon from '@mui/icons-material/ArticleOutlined';
import FolderOpenOutlinedIcon from '@mui/icons-material/FolderOpenOutlined';
import FolderOutlinedIcon from '@mui/icons-material/FolderOutlined';
import TreeItem, { treeItemClasses } from '@mui/lab/TreeItem';
import TreeView from '@mui/lab/TreeView';
import { Alert, Box, Grid, Link } from '@mui/material';
import Typography from '@mui/material/Typography';
import { styled } from '@mui/material/styles';
import MarkdownPreview from '@uiw/react-markdown-preview';
import React, { Fragment, useEffect, useState } from 'react';
import { Link as RouterLink } from 'react-router-dom';
import config from '../../config.json';
import { Progress } from '../library/loading';
import { docsMdReads, docsMenu } from '../rest/docs';
import { useSearchChange } from '../utils/hook';
//填充菜单中ID
function generateId(node, pids, menu) {
    if (!node) return {};
    if (!node.id) {
        node.id = (pids ? (pids + "-") : "") + node.path ?? '未知';
    }
    node.menu = menu;
    if (node.children) {
        for (let child of node.children) {
            generateId(child, (pids ? (pids + "-") : "") + node.id, menu);
        }
    }
}
//合并多个菜单
function mergeJSONs(json1, json2) {
    if ((!json1.children || json1.children.length == 0)
        && (!json2.children || json2.children.length == 0)) {
        if (json1.menu?.id>json2.menu?.id) return json1
        else return json2
    }
    if (!json1.children) {
        json1.children = []
    }
    if (!json2.children) {
        json2.children = []
    }
    const mergedChildren = [];

    json1.children.forEach(child1 => {
        const matchingChild = json2.children.find(child2 => child2.id === child1.id);
        if (matchingChild) {
            mergedChildren.push(mergeJSONs(child1, matchingChild));
        } else {
            mergedChildren.push(child1);
        }
    });

    json2.children.forEach(child2 => {
        const matchingChild = json1.children.find(child1 => child1.id === child2.id);

        if (!matchingChild) {
            mergedChildren.push(child2);
        }
    });
    return {
        ...(json1.menu?.id>json2.menu?.id)?json1:json2,
        name: (json1.name && json1.name != '') ? json1.name : json2.name,
        children: mergedChildren
    };
}

//获取第一个文档
function findFirstDOC(jsondata) {
    if (jsondata.path) return jsondata
    if (jsondata.children) {
        for (const e of jsondata.children) {
            let f = findFirstDOC(e)
            if (f) return f
        }
    }
}
//查找打开菜单节点
function findNowOpenNode(muneData,id,path,parent) {
    for (var jsondata of muneData){
        let mpath=((jsondata?.path?jsondata?.path:'')+'').replace(/^\.\//,'').replace(/^\//,'');
        let spath=(path?path:'').replace(/^\.\//,'').replace(/^\//,'');
       
        if(jsondata?.menu?.id==id&&mpath==spath){
            parent.push(jsondata.id)
            return parent
       }
       if (jsondata.children) {
           let tmp=[...parent];
           tmp.push(jsondata.id)
           let f = findNowOpenNode(jsondata.children,id,path,tmp)
           if (f.length>0) return f
       }
    }
    return []
}
//文档路径是否非URL地址
function needChangePath(path) {
    return !(
        /^(http|https|ftp|file|data|ws|wss|blob):\/\/.+/.test(path)
        || /^\[a-z]:/.test(path)
    )
}
//替换文件路径,图片
function docFileSrc(id, file) {
    let url = `${config.serverURL}/api/docs/raw/${id}/${file}`;
    return url;
}

const StyledTreeItemRoot = styled(TreeItem)(({ theme }) => ({
    color: theme.palette.text.secondary,
    [`& .${treeItemClasses.content}`]: {
        color: theme.palette.text.secondary,
       
        fontWeight: theme.typography.fontWeightMedium,
        '&.Mui-expanded': {
            fontWeight: theme.typography.fontWeightRegular,
        },
        '&:hover': {
            backgroundColor: theme.palette.action.hover,
        },
        [`& .${treeItemClasses.label}`]: {
            fontWeight: 'inherit',
            color: 'inherit',
        },
    },
    [`& .${treeItemClasses.group}`]: {
        marginLeft: 0,
        [`& .${treeItemClasses.content}`]: {
            paddingLeft: theme.spacing(2),
        },
    },
}));

function StyledTreeItem(props) {
    const {
        bgColor,
        color,
        labelIcon: LabelIcon,
        labelInfo,
        labelText,
        to,
        ...other
    } = props;

    let label=<Box sx={{ display: 'flex', alignItems: 'center', p: 0.7, pl:0, pr: 0 }}>
        <Box fontSize={"small"} component={LabelIcon} color="inherit" sx={{ mr: 1 }} />
        <Typography variant="body2" sx={{ fontWeight: 'inherit', flexGrow: 1 }}>
            {labelText}
        </Typography>
        <Typography variant="caption" color="inherit">
            {labelInfo}
        </Typography>
    </Box>;
    if(to){
        label= <Link underline="none" sx={{
            display: "block"
        }} component={RouterLink} to={to} >{label}</Link>
    }
    return (
        
        <StyledTreeItemRoot
            label={label}
            style={{
                '--tree-view-color': color,
                '--tree-view-bg-color': bgColor,
            }}
            {...other}
        />
    );
}

function TreeMenuView(props) {
    const { data,...other } = props
    const renderTree = (nodes) => {return (
        
        <StyledTreeItem to={
            nodes.path?`?id=${nodes.menu.id}&path=${nodes.path}`:null
        } sx={{ pb: 1 }} key={nodes.id} 
        nodeId={nodes.id} 
        labelIcon={nodes.children&&nodes.children.length?null:ArticleOutlinedIcon} 
        labelText={
            nodes.name ?? nodes.path
        } title={nodes.title ?? null} >
            {Array.isArray(nodes.children) && nodes.children
                ? nodes.children.map((node) => renderTree(node))
                : null}
        </StyledTreeItem>
    )};

    
    
    return <TreeView
    
        defaultCollapseIcon={<FolderOpenOutlinedIcon />}
        defaultExpanded={['root']}
        defaultExpandIcon={<FolderOutlinedIcon />}
        {...other}
    >
        {data.map((item) => renderTree(item))}
    </TreeView>;
}



export default function DocPage() {
    let [menuData, setMenuData] = useState({
        loading: true,
        status: true,
        data: [],
        message: null,
    });

    const [expanded, setExpanded] = React.useState([]);
    const [menuSelect, setMenuSelect] = React.useState("");


    const handleToggle = (event, nodeIds) => {
        setExpanded(Array.isArray(nodeIds)?nodeIds:[nodeIds]);
    };

    const handleSelect = (event,nodeIds ) => {
       setMenuSelect(nodeIds);
    };

    const initMenuSelect=(menuData,id,path)=>{
        if (!id || !path)return ;
        let sitem=findNowOpenNode(menuData,id,path,[]);
        if (sitem.length==0)return;
        setExpanded(sitem)
        setMenuSelect(sitem.pop()  )
    }
    


    const loadMenuData = () => {
        setMenuData({
            ...menuData,
            data: [],
            loading: true
        })
        docsMenu().then((data) => {
            let mdata = (data.data ?? []).reduce((out, add) => {
                if (!add.menu_data) return out;
                let madd = { ...add.menu_data };
                if ((!madd.children || madd.children.length == 0)
                    && (!madd.path || madd.path == '')) return out;
                generateId(madd, "", {
                    id: add.id,
                    tag_id: add.tag_id,
                    version: add.version,
                    path: add.menu_path,
                })
                if (!madd.id) return out;
                let find = false;
                let rout = [...out].map((tmp) => {
                    if (tmp.id == madd.id) {
                        find = true
                        tmp = mergeJSONs(tmp, madd)
                    }
                    return tmp
                })
                if (!find) rout.push(madd)
                return rout
            }, []);
            let msg = data.message;
            if (data.status && mdata.length == 0) {
                msg = "系统还未配置文档,请稍后查看。"
                data.status = false
            }
            if (!data.status) {
                setMenuData({
                    ...menuData,
                    loading: false,
                    status: false,
                    message: msg
                })
                return;
            }
            setMenuData({
                ...menuData,
                data: mdata,
                loading: false,
                status: true,
            })

            initMenuSelect(mdata,searchParam.get("id"),searchParam.get("path"));

        })
    }

    


    let [docData, setDocData] = useState({
        abort: null,
        loading: true,
        status: true,
        data: '',
        menu_id: '',
        path: '',
        message: null,
    });
    const [searchParam, setSearchParam] = useSearchChange({
        id: "",
        path: '',
        page_size: 25,
    });
    useEffect(() => {
        loadMenuData()
    }, [])
    const checkInitPage = () => {
        if (searchParam.get("id") && searchParam.get("path")) return false;
        let setData = null;
        if (menuData.data) {
            for (const e of menuData.data) {
                setData = findFirstDOC(e)
                break;
            }
        }
        if (!setData) return;
        setSearchParam({
            id: setData.menu.id,
            path: setData.path,
        })
        return true;
    }
    useEffect(() => {
        checkInitPage();
    }, [menuData.data])
    useEffect(() => {
        if (checkInitPage()) return;
        if (!searchParam.get("id") || !searchParam.get("path")) return;
        if (docData.abort) {
            docData.abort.abort()
        }
        docData.abort = new AbortController();
        setDocData({
            ...docData,
            loading: true,
        })
        let req_param = {
            menu_id: searchParam.get("id"),
            url: searchParam.get("path"),
        };

        initMenuSelect(menuData.data,req_param.menu_id,req_param.url);
           
        docsMdReads(req_param, {
            signal: docData.abort.signal
        }).then((data) => {
            if (!data.status) {
                setDocData({
                    ...docData,
                    loading: false,
                    status: false,
                    message: data.message ?? "加载GIT异常"
                })
                return;
            }
            let path = req_param.url.split("/");
            path.length > 0 && path.pop();
            path = path.join("/")
            setDocData({
                ...docData,
                menu_id: req_param.menu_id,
                path: path == '/' ? '' : path,
                data: data.data ?? '',
                loading: false,
                status: true,
                message: null,
            })

            

        })
    }, [searchParam])


    return  <Fragment>
        {
            menuData.loading ? <Progress /> :
                !menuData.status ?
                    <Alert sx={{ m: 3, width: 1 }} severity="error">{menuData.message}</Alert> :
                   
                         <Grid
                            container
                            direction="row"
                            justifyContent="space-between"
                            alignItems="stretch"
                            spacing={1}
                            sx={{flexWrap:"nowrap"}}
                            >
                                <Grid item xs={2}  >
                                    <TreeMenuView 
                                        sx={{m:2,minWidth:220}} 
                                        data={menuData.data} 
                                        expanded={expanded}
                                        selected={menuSelect}
                                        onNodeToggle={handleToggle}
                                        onNodeSelect={handleSelect}

                                    />
                                </Grid>
                                <Grid item xs={10} >
                      
                            {docData.loading ? <Progress /> : null}
                            {docData.message ? <Alert sx={{ m: 3 }} severity="error">{docData.message}</Alert> : null}
                            {docData.status ? <MarkdownPreview
                                style={{ padding: 24 }}
                                wrapperElement={{
                                    "data-color-mode": "light"
                                }}
                                rehypeRewrite={docData.menu_id > 0 ? (node, index, parent) => {

                                    switch (node.tagName) {
                                        case 'img':
                                            if (needChangePath(node.properties.src ?? '')) {
                                                let src = node.properties.src;
                                                if (/^\//.test(src)) {
                                                    src = src.substr(1);
                                                } else {
                                                    src = (docData.path && docData.path.length > 0 ? (docData.path + '/') : '') + src;
                                                }
                                                node.properties.src = docFileSrc(docData.menu_id, src)
                                            }
                                            break;
                                        case 'a':
                                            if (needChangePath(node.properties.href ?? '')) {
                                                let href = node.properties.href;
                                                if (/^\//.test(href)) {
                                                    href = href.substr(1);
                                                } else {
                                                    if ((docData.path + '').indexOf("/") > 0) {
                                                        src = docData.path + '/' + href;
                                                    }
                                                }
                                                node.properties.href = `#/doc?id=${docData.menu_id}&path=` + href
                                            }
                                            break;
                                    }
                                } : null}
                                source={docData.data} /> : null}
                        </Grid> 
                        </Grid> 
                       
        }
    </Fragment >
        ;
}


