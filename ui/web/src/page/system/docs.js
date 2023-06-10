import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import AutoModeOutlinedIcon from '@mui/icons-material/AutoModeOutlined';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';

import ReadMoreOutlinedIcon from '@mui/icons-material/ReadMoreOutlined';
import { Alert, Button, Divider, Drawer, FormControl, FormControlLabel, FormLabel, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Stack, Switch, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../context/toast';
import { ConfirmButton } from '../../library/dialog';
import { LoadingButton } from '../../library/loading';
import { DataTablePage } from '../../library/table_page';
import { docsAdd, docsGitDetail, docsList } from '../../rest/docs';
import { showTime } from '../../utils/utils';
import { PageNav } from './menu';




export function AddBox(props) {
    const { toast } = useContext(ToastContext);
    const [addData, setaddData] = useState({
        url: '',
        branch:'',
        is_tag:'',
        is_update:true,
        version:'',
        menu_data:[{
            menu_path:"/menu.json",
            access_path:"/",
            clean_rule:["*.js$","*.rs$","*.css$"]
        },{
            menu_path:"/menu1.json",
            access_path:"/",
            clean_rule:["*.js$","*.rs$","*.css$"]
        },{
            menu_path:"/menu2.json",
            access_path:"/",
            clean_rule:["*.js$","*.rs$","*.css$"]
        }],
    });
    const [menuData, setMenuData] = useState({
        menu_path:"",
        access_path:"",
        clean_rule:[]
    });
    const [addError, setAddError] = useState({
        url: '',
        branch:'',
        version:''
    });
    const [loadData, setLoadData] = useState({
        git_data: null,
        loading: false,
    });
    const doGit = function () {
        setLoadData({
            ...loadData,
            loading: true
        })
        docsGitDetail({
            url: addData.url+'',
        }).then((data) => {
            if (data.status&&(
                !data.data||
                data.data.length==0
            )){
                data.status=false;
                data.message="未发现分支或标记";
            }
            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setLoadData({
                    ...loadData,
                    loading: false
                })
            } else {
                setLoadData({
                    ...loadData,
                    git_data: data.data,
                    loading: false
                })
                setaddData({
                    ...addData,
                    is_tag: data.data[0].is_tag,
                   branch:data.data[0].branch,
                   version:data.data[0].version,
                })
            }
        })
    };
    const doAdd = function () {
        setLoadData({
            ...loadData,
            loading: true
        })
        docsAdd({
            url:addData.url+'',
            branch:addData.branch+'',
            version: addData.version+'',
            is_tag:addData.is_tag,
            is_update:addData.is_update,
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setLoadData({
                    ...loadData,
                    loading: false
                })
            } else {
                setLoadData({
                    ...loadData,
                    git_data: data.data,
                    loading: false
                })
                setaddData({
                    ...addData,
                    is_tag: data.data[0].is_tag,
                   branch:data.data[0].branch,
                   version:data.data[0].version,
                })
            }
        })
    };

    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                添加Git文档
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
               
            }}>
                <Stack >

                </Stack>
                <Grid
                    sx={{
                        mt: 5,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`Git文档地址`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setaddData({
                                    ...addData,
                                    url: e.target.value
                                })
                            }}
                            value={addData.url}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.url}
                            helperText={addError.url}
                        />
                    </Grid>
                    {loadData.git_data? <Fragment>
                    <Grid item xs={10}> 
                    <FormControl  fullWidth >
                    <InputLabel size="small" id="user-select-label">选择分支或Tag</InputLabel>
                    <Select
                        size='small'
                        fullWidth
                        sx={{
                            width: 1,
                            mb: 2
                        }}
                        label="选择分支或Tag"
                        value={addData.is_tag+'-'+addData.branch}
                        onChange={(e)=>{
                            loadData.git_data.map((item) => {
                                if (item.is_tag+'-'+item.branch==e.target.value){
                                    setaddData({
                                        ...addData,
                                        is_tag:item.is_tag,
                                        branch:item.branch,
                                        version:item.version
                                    })
                                }
                            })
                        }}
                    >
                        {
                            loadData.git_data.map((item) => {
                                return <MenuItem key={`git-b-${item.version}`} value={item.is_tag+'-'+item.branch}>{item.is_tag?`Tag:${item.branch}`:`Branch:${item.branch}`}</MenuItem>;
                            })
                        }
                    </Select>
                    </FormControl>
                    </Grid>
                    <Grid item xs={10}>
                        <Stack direction={"row"} spacing={1}>
                        <TextField
                            disabled={addData.is_update}
                            variant="outlined"
                            label={`使用版本`}
                            type="text"
                            size="small"
                           
                            onChange={(e) => {
                                setaddData({
                                    ...addData,
                                    version: e.target.value
                                })
                            }}
                            value={addData.version}
                            sx={{
                                width: 1,
                                mb: 2,
                            }}
                            error={!!addError.version}
                            helperText={addError.version}
                        />
                        <FormControlLabel
                         size="small"
                            label={<span style={{fontSize:"0.7rem",color: "#666"}}>自动更新</span>}
                            sx={{width:140 ,height:40}}
                        
                            control={
                                <Switch
                                    size="small"
                                    value={addData.is_update?'0':'1'}
                                    onChange={(e) => {
                                        setaddData({
                                            ...addData,
                                            is_update: e.target.value=='1'
                                        })
                                    }} />
                            }
                        />
                        </Stack>
                    </Grid>
                    <Grid item xs={10}> 
                    <FormControl  fullWidth sx={{ mb:2}} >
                    <FormLabel style={{
                        position: "absolute",
                        transform: "translate(10px, -5px) scale(0.75)"
                    }}>文档目录</FormLabel>
                    <Box className='MuiInputBase-root MuiOutlinedInput-root MuiInputBase-colorPrimary MuiInputBase-formControl MuiInputBase-sizeSmall'
                        style={{
                            borderRadius: "4px"
                        }}>
                        <fieldset style={{
                            textAlign: "left",
                            position: "absolute",
                            bottom: 0,
                            right: 0,
                            top: "-5px",
                            left: 0,
                            margin: 0,
                            padding: "0 8px",
                            pointerEvents: "none",
                            borderRadius: "inherit",
                            borderStyle: "solid",
                            borderWidth: "1px ",
                            overflow: "hidden",
                            borderColor: " rgba(0, 0, 0, 0.23)",
                        }} className="MuiOutlinedInput-notchedOutline "><legend style={{
                            visibility: "hidden"
                        }} ><span>权限操作</span></legend></fieldset>
                    </Box>
                    <Box sx={{pt:3}}>
                    {addData.menu_data.length?<Fragment>
                        {
                            addData.menu_data.map((item,i)=>{
                                return   <Box key={`m-d${i}`} sx={{
                                    border: "1px solid #eee",
                                    margin: "0 8px 8px 8px",
                                    p: 1,
                                    borderRadius: "4px"
                                }}>
                                    <Stack direction={"row"} sx={{
                                        position: "absolute",
                                        right:16
                                    }}>
                                        <IconButton title="删除" fontSize="small" onClick={()=>{
                                            setaddData({
                                                ...addData,
                                                menu_data: addData.menu_data.filter((_,ii)=>{
                                                 return    i===ii
                                                })
                                            })
                                           

                                            setMenuData({
                                                ...menuData,
                                                menu_path: item.menu_path,
                                                access_path: item.access_path,
                                                clean_rule: item.clean_rule??[],
                                            })
                                        }}><EditIcon  fontSize="small"/> </IconButton>
                                        <IconButton title="删除" fontSize="small"><DeleteIcon  fontSize="small"/> </IconButton>
                                    
                                    </Stack>
                                    <Typography variant="subtitle2">
                                        目录路径:<span style={{marginLeft:4}}>{item.menu_path}</span><br/>
                                        访问限制路径:<span style={{marginLeft:4}}>{item.access_path}</span>
                                    </Typography>
                                        {item.clean_rule&&item.clean_rule.length?  <Typography variant="caption">
                                    克隆后删除规则:
                                    {item.clean_rule.map((ritem)=>{
                                        return <span key={`rule-${ritem}`} style={{border:"1px solid #ccc",borderRadius:4,padding:4,margin:4}}>{ritem}</span>
                                    })}
                                  </Typography>:null}
                                    
                                  
                                </Box>
                            })
                        } 
                      
                    </Fragment>:
                        <Box sx={{mt: 1,lineHeight:3, fontSize:"0.75rem",color: "#999",textAlign:"center" }}>请添加目录文件路径</Box>}
                    </Box>
                    <Divider sx={{ mt: 1 }}></Divider>
                    <Stack sx={{m:1,p:1}}>
                         <TextField
                            sx={{ mb:1}}
                            placeholder='/menu.json'
                            variant="outlined"
                            label={`目录路径`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setMenuData({
                                    ...menuData,
                                    menu_path: e.target.value.replace(/^\s+/,'').replace(/\s+$/,'')
                                })
                            }}
                            value={menuData.menu_path}
                    
                        />
                        <TextField
                            sx={{ mb:1}}
                            placeholder='/'
                            variant="outlined"
                            label={`访问限制路径`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setMenuData({
                                    ...menuData,
                                    access_path: e.target.value.replace(/^\s+/,'').replace(/\s+$/,'')
                                })
                            }}
                            value={menuData.access_path}
                        />
                        <TextField
                            multiline
                            row={2}
                            maxRows={3}
                            sx={{ mb:1}}
                            variant="outlined"
                            label={`克隆后删除规则`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                               let tt=(e.target.value+'').split("\r").map((t)=>{t.replace(/^\s+/,'').replace(/\s+$/,'')}).filter((e)=>e)
                                setMenuData({
                                    ...menuData,
                                    clean_rule: tt
                                })
                            }}
                            value={menuData.clean_rule.join("\r")}
                        />
                        <Button onClick={()=>{
                             
                             setaddData({
                                ...addData,
                                menu_data: [...addData.menu_data,{
                                    clean_rule:menuData.clean_rule,
                                    menu_path:menuData.menu_path,
                                    access_path:menuData.access_path,
                                }]
                            })
                           
                        }} sx={{
                            width: 1,
                        }} variant="contained" type="submit">添加目录</Button>
                    </Stack>
                    </FormControl>
                    </Grid>
                    {addData.menu_data.length? <Grid item xs={10}  >
                        <LoadingButton disabled={loadData.loading} loading={loadData.loading} onClick={()=>{
                            doAdd()
                        }} sx={{
                            width: 1,
                            mb:2
                        }} variant="contained" type="submit">添加</LoadingButton>
                    </Grid>:null}
                   
                    </Fragment>: <Grid item xs={10}>
                        <LoadingButton disabled={loadData.loading} loading={loadData.loading} onClick={()=>{
                        doGit()
                        }} sx={{
                            width: 1,
                        }} variant="contained" type="submit">查询</LoadingButton>
                    </Grid>}
                   
                   
                </Grid>
            </Form >
        </Fragment>)
}



export default function SystemDocsPage(props) {

    //列表数据
    let [loadData, setLoadData] = useState({
        loading: true,
        status: false,
        data: [],
        message: null,
    });
    const loadDocsData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        docsList({}).then((data) => {
            setLoadData({
                ...loadData,
                ...data,
                data: data.status ? data.data : [],
                loading: false
            })
        })
    };
    useEffect(loadDocsData, [])

    //添加跟更新
    const [changeBoxState, setChangeBox] = useState(0);
    const { toast } = useContext(ToastContext);
    //add 
    const [addData, setAddData] = useState({
        name: '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
    });
    const doAdd = function () {
        setAddData({
            ...addData,
            loading: true
        })
        docsList({}).then((data) => {

            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setAddData({
                    ...addData,
                    loading: false,
                })
            } else {
                showCodeBox(data.id, addData.name)
                setAddData({
                    ...addData,
                    name: '',
                    loading: false,
                })
                setParam({
                    page: 0,
                    status: comfirmStatus[0].key
                })
            }
        })
    };


    const columns = [
        {
            field: 'id',
            label: 'ID',
            style: { width: 90 },
            align: "right",
        },
        {
            field: 'email',
            style: { width: 260 },
            label: 'URL',
        },
        {

            style: { width: 140 },
            label: '使用分支',
            render: (row) => {
                return 'xxxx32';
            }
        },
        {
            style: { width: 160 },
            label: '当前构建版本',
            render: (row) => {
                return <Stack direction="row" spacing={1}>
                    <Box>dd23423</Box>
                    <Box><AutoModeOutlinedIcon color='secondary' fontSize='small' /></Box>
                </Stack>;
            }
        },
        {
            style: { width: 170 },
            label: '构建时间',
            render: (row) => {
                return showTime(row.confirm_time, "未构建")
            }
        },
        {
            style: { width: 170 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            label: '操作',
            render: (row) => {
                return <Fragment>
                    <IconButton title="文档详细" key={`${row.id}-send`} onClick={() => {
                        showCodeBox(row.id, row.email)
                    }} size='small'>
                        <ReadMoreOutlinedIcon fontSize='small' />
                    </IconButton>
                    <ConfirmButton
                        key={`${row.id}-confirm`}
                        message={`确定要删除此文档路径 [${row.id}] ?`}
                        onAction={() => {
                            // return emailDelete(row.id).then((res) => {
                            //     if (!res.status) return res;
                            //     let rows = loadData.data.filter((item) => {
                            //         if (item.id != row.id) return item;
                            //     })
                            //     setLoadData({
                            //         ...loadData,
                            //         data: rows
                            //     })
                            //     toast("删除完成");
                            //     if (rows.length == 0) {
                            //         loadEmailData()
                            //         setParam({
                            //             page: param.get("page") - 1 >= 0 ? param.get("page") : 0,
                            //         });
                            //     }
                            //     return res;
                            // });
                        }}
                        renderButton={(props) => {
                            return <IconButton title="删除" key={`${row.id}-sms`} {...props} size='small'>
                                <DeleteIcon fontSize='small' />
                            </IconButton>
                        }} />
                </Fragment>
            }
        },
    ];

    let showBox
    switch (changeBoxState) {
        case 1:
            showBox = <AddBox
                title="添加邮箱"
                type="email"
                label="邮箱"
                placeholder="输入新邮箱"
                button="添加"
                onSubmit={() => {
                    doAdd()
                }}
                onChange={(e) => {
                    setAddData({
                        ...addData,
                        name: e.target.value
                    })
                    setAddError({
                        ...addError,
                        name: ''
                    })
                }}
                name={addData.name}
                nameError={addError.name}
                loading={addData.loading}
            />;
            break
        case 2:
            showBox = <CodeBox
                label="邮箱"
                title="添加邮箱"
                button="添加"
                onSubmit={doCode}
                onChange={(e) => {
                    setCodeData({
                        ...codeData,
                        captcha_val: e.target.value
                    })
                    setCodeError({
                        ...codeError,
                        captcha: ''
                    })
                }}
                name={codeData.name}
                codeError={codeError.captcha}
                code={codeData.captcha_val}
                loading={codeData.loading}
                captchaSrc={codeData.captcha_src}
            />
            break

    };


    return <Fragment>
        <PageNav />
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState != 0}
            onClose={() => {
                setChangeBox(0)
            }}
        >
            <Box
                sx={{ width: 450 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox(1)
                }}>
                添加Git文档
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <DataTablePage
                    rows={(loadData.data ?? [])}
                    columns={columns}
                    loading={loadData.loading}
                />

            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}