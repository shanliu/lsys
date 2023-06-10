import DeleteIcon from '@mui/icons-material/Delete';
import { Alert, Backdrop, Box, Grid, IconButton, LinearProgress, MenuItem, Paper, Select, TableFooter, TablePagination, Typography } from '@mui/material';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import PropTypes from 'prop-types';
import React, { Fragment, useEffect, useState } from 'react';
import { useTheme } from '@emotion/react';
import FirstPageIcon from '@mui/icons-material/FirstPage';
import KeyboardArrowLeft from '@mui/icons-material/KeyboardArrowLeft';
import KeyboardArrowRight from '@mui/icons-material/KeyboardArrowRight';
import LastPageIcon from '@mui/icons-material/LastPage';
import { Progress } from './loading';

//表中无数据默认显示
export function BaseTableNoRows(props) {
    const { msg } = props;
    return <div style={
        {
            textAlign: "center",
            lineHeight: "54px",
            color: "#999"
        }
    }>{msg ? msg : '无结果'}</div>
}

BaseTableNoRows.propTypes = {
    msg: PropTypes.string
};


//公用表标题
export function BaseTableHead(props) {
    const {
        columns,
    } = props;
    return <TableHead>
        <TableRow>
            {
                columns.map((column, index) => {
                    const { label, field, render, ...column_porps } = column;
                    return <TableCell key={`column-${index}`} {...column_porps} >{label}</TableCell>;
                })
            }
        </TableRow>
    </TableHead>
}

BaseTableHead.propTypes = {
    columns: PropTypes.array.isRequired
};


//公用表数据块
export function BaseTableBody(props) {
    const {
        rows,
        columns,
        loading,
        cellProps,
    } = props;
    let load_rows;
    let not_rows;
    if (loading) {
        let tips_rows;
        if (!rows || rows.length == 0) {
            tips_rows = <div style={
                {
                    textAlign: "center",
                    lineHeight: "54px"
                }
            }>数据加载中...</div>
        }
        load_rows = <TableRow key={`load-rows`}>
            <TableCell key={`load-rows-cell`} sx={{ padding: 0 }} colSpan={columns.length}>
                <Progress />
                {tips_rows}
            </TableCell>
        </TableRow >
    } else {
        if (!rows || rows.length == 0) {
            not_rows = <TableRow key={`not-rows`}>
                <TableCell key={`not-rows-cell`} colSpan={columns.length} {...cellProps}>
                    <BaseTableNoRows />
                </TableCell>
            </TableRow>;
        }
    }
    return <TableBody>
        {load_rows}
        {not_rows}
        {
            rows ? rows.map((row, ri) => {
                return <BaseTableBodyRow row={row} columns={columns} key={`key-${ri}`} hover />
            }) : null
        }
    </TableBody>
}
BaseTableBody.defaultProps = {
    loading: false,
    rows: []
};
BaseTableBody.propTypes = {
    columns: PropTypes.array.isRequired,
    loading: PropTypes.bool,
    row: PropTypes.array,
};


//公用表记录块
export function BaseTableBodyRow(props) {
    const {
        row,
        columns,
        hover,

    } = props;
    return <TableRow hover={hover}>
        {
            columns.map((column, ci) => {
                const { label, field, render, ...column_porps } = column;
                let val = '';
                if (typeof field == 'string') {
                    val = row[field] ?? '';
                }
                if (render && typeof render == 'function') {
                    val = render(row);
                }
                return <TableCell key={`tmp-${ci}`} {...column_porps}>{val}</TableCell>;
            })
        }
    </TableRow>
}
BaseTableBodyRow.defaultProps = {
    row: {}
};
BaseTableBodyRow.propTypes = {
    columns: PropTypes.array.isRequired,
    row: PropTypes.object,
    hover: PropTypes.bool
};





//公用表底部


function TablePaginationActions(props) {
    const theme = useTheme();
    const { count, page, rowsPerPage, onPageChange } = props;

    const handleFirstPageButtonClick = (event) => {
        onPageChange(event, 0);
    };

    const handleBackButtonClick = (event) => {
        onPageChange(event, page - 1);
    };

    const handleNextButtonClick = (event) => {
        onPageChange(event, page + 1);
    };

    const handleLastPageButtonClick = (event) => {
        onPageChange(event, Math.max(0, Math.ceil(count / rowsPerPage) - 1));
    };

    return (
        <Box sx={{ flexShrink: 0, ml: 2.5 }}>
            <IconButton
                onClick={handleFirstPageButtonClick}
                disabled={page === 0}

            >
                {theme.direction === 'rtl' ? <LastPageIcon /> : <FirstPageIcon />}
            </IconButton>
            <IconButton
                onClick={handleBackButtonClick}
                disabled={page === 0}

            >
                {theme.direction === 'rtl' ? <KeyboardArrowRight /> : <KeyboardArrowLeft />}
            </IconButton>
            <IconButton
                onClick={handleNextButtonClick}
                disabled={page >= Math.ceil(count / rowsPerPage) - 1}

            >
                {theme.direction === 'rtl' ? <KeyboardArrowLeft /> : <KeyboardArrowRight />}
            </IconButton>
            <IconButton
                onClick={handleLastPageButtonClick}
                disabled={page >= Math.ceil(count / rowsPerPage) - 1}

            >
                {theme.direction === 'rtl' ? <FirstPageIcon /> : <LastPageIcon />}
            </IconButton>
        </Box>
    );
}

TablePaginationActions.propTypes = {
    count: PropTypes.number.isRequired,
    onPageChange: PropTypes.func.isRequired,
    page: PropTypes.number.isRequired,
    rowsPerPage: PropTypes.number.isRequired,
};
export function BaseTableFooter(props) {
    const {
        count,
        page,
        rowsPerPage,
        onPageChange,
        onRowsPerPageChange,
    } = props;
    let countNum = parseInt(count) >= 0 ? parseInt(count) : 0;
    let pageNum = parseInt(page) >= 0 ? parseInt(page) : 0;
    let rowsPerPageNum = parseInt(rowsPerPage) >= 0 ? parseInt(rowsPerPage) : 0;
    let pageCom;
    if (countNum > 0) {
        pageCom = <TableFooter>
            <TableRow>
                <TablePagination
                    labelRowsPerPage={`分页数`}
                    labelDisplayedRows={({ from, to, count }) => {
                        return `${from}-${to} 总计: ${count !== -1 ? count : `超过 ${to}`}`;
                    }}
                    count={countNum}
                    page={pageNum}
                    rowsPerPage={rowsPerPageNum}
                    onPageChange={onPageChange}
                    onRowsPerPageChange={onRowsPerPageChange}
                    ActionsComponent={TablePaginationActions}
                />
            </TableRow>
        </TableFooter>;
    }
    return pageCom
}


BaseTableFooter.propTypes = {
    count: PropTypes.oneOfType([
        PropTypes.number.isRequired,
        PropTypes.string.isRequired
    ]),
    page: PropTypes.oneOfType([
        PropTypes.number.isRequired,
        PropTypes.string.isRequired
    ]),
    rowsPerPage: PropTypes.oneOfType([
        PropTypes.number.isRequired,
        PropTypes.string.isRequired
    ]),
    onPageChange: PropTypes.func.isRequired,
    onRowsPerPageChange: PropTypes.func.isRequired,
};


//公用表显示

export function SimpleTablePage(props) {
    const {
        count,
        page,
        rowsPerPage,
        onPageChange,
        onRowsPerPageChange,
        rows,
        columns,
        loading
    } = props;
    return <TableContainer component={Paper}>
        <Table>
            <BaseTableHead
                columns={columns}
            />
            <BaseTableBody
                columns={columns}
                loading={loading}
                rows={rows}
            />
            <BaseTableFooter
                count={count}
                page={page}
                rowsPerPage={rowsPerPage}
                onPageChange={onPageChange}
                onRowsPerPageChange={onRowsPerPageChange}
            />
        </Table>
    </TableContainer>
}




export function SimplePaginationTableFooter(props) {
    const {
        isFirst,//是否首页
        isEnd,//下一页开始值
        rowsPerPage,//分页数量
        onPageChange,//(e, bool)
        onRowsPerPageChange,
    } = props;
    const theme = useTheme();
    return <TableFooter>
        <TableRow>
            <TableCell colSpan={1000} >

                <Grid

                    className="MuiToolbar-root MuiToolbar-gutters MuiToolbar-regular MuiTablePagination-toolbar"
                    container
                    direction="row"
                    justifyContent="flex-end"
                    alignItems="center"
                >

                    <Box
                        sx={{

                            width: 180,
                            display: "flex",
                            direction: "row",
                            justifyContent: "flex-end",
                            alignItems: "center",
                            mr: 2
                        }}
                    >
                        <Typography sx={{
                            fontSize: "0.9rem",
                            mr: 2
                        }} className='MuiTablePagination-selectLabel'>分页数:</Typography>
                        <Select
                            disableUnderline
                            variant="standard"
                            size='small'
                            value={rowsPerPage}
                            inputProps={{
                                sx: {
                                    fontSize: "0.9rem",
                                    paddingTop: "4px",
                                    paddingLeft: "4px",
                                }
                            }}
                            onChange={onRowsPerPageChange}
                        >
                            <MenuItem value={10}>10</MenuItem>
                            <MenuItem value={20}>20</MenuItem>
                            <MenuItem value={50}>50</MenuItem>
                            <MenuItem value={100}>100</MenuItem>
                        </Select>
                    </Box>
                    <Box className="MuiBox-root">
                        <IconButton
                            onClick={(e) => {
                                onPageChange(e, false)
                            }}
                            disabled={isFirst}
                        >
                            {theme.direction === 'rtl' ? <KeyboardArrowRight /> : <KeyboardArrowLeft />}
                        </IconButton>
                        <IconButton
                            onClick={(e) => {
                                onPageChange(e, true)
                            }}
                            disabled={isEnd}
                        >
                            {theme.direction === 'rtl' ? <KeyboardArrowLeft /> : <KeyboardArrowRight />}
                        </IconButton>
                    </Box>

                </Grid>

            </TableCell>
        </TableRow>
    </TableFooter >;
}



export function SimplePaginationTablePage(props) {
    const {
        isFirst,//是否首页
        isEnd,//最后一页
        rowsPerPage,
        onPageChange,//(e, newPage)
        onRowsPerPageChange,
        rows,
        columns,
        loading
    } = props;
    return <TableContainer component={Paper}>
        <Table>
            <BaseTableHead
                columns={columns}
            />
            <BaseTableBody
                columns={columns}
                loading={loading}
                rows={rows}
            />
            <SimplePaginationTableFooter
                isEnd={isEnd}
                isFirst={isFirst}
                rowsPerPage={rowsPerPage}
                onPageChange={onPageChange}
                onRowsPerPageChange={onRowsPerPageChange}
            />
        </Table>
    </TableContainer>
}

//公用表显示

export function DataPaginationTablePage(props) {
    const {
        page,
        rowsPerPage,
        onPageChange,
        onRowsPerPageChange,
        rows,
        columns,
        loading
    } = props;

    return <TableContainer component={Paper}>
        <Table>
            <BaseTableHead
                columns={columns}
            />

            {rows && rows.length > 0 ? <BaseTableBody
                columns={columns}
                loading={loading}
                rows={rowsPerPage > 0
                    ? rows.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
                    : rows}
            /> : <TableBody><TableRow key={`not-rows`}>
                <TableCell key={`not-rows-cell`} colSpan={columns.length} >
                    <BaseTableNoRows />
                </TableCell>
            </TableRow></TableBody>}


            <BaseTableFooter
                count={rows.length ?? 0}
                page={page}
                rowsPerPage={rowsPerPage}
                onPageChange={onPageChange}
                onRowsPerPageChange={onRowsPerPageChange}
            />


        </Table>
    </TableContainer >

}


//公用表显示

export function DataTablePage(props) {
    const {
        rows,
        columns,
        loading
    } = props;

    return <TableContainer component={Paper}>
        <Table>
            <BaseTableHead
                columns={columns}
            />

            {rows && rows.length > 0 ? <BaseTableBody
                columns={columns}
                loading={loading}
                rows={rows}
            /> : <TableBody><TableRow key={`not-rows`}>
                <TableCell key={`not-rows-cell`} colSpan={columns.length} >
                    <BaseTableNoRows />
                </TableCell>
            </TableRow></TableBody>}
        </Table>
    </TableContainer >

}
