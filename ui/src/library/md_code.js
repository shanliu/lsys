import React, {
    useState,
    useRef,
    useEffect,
    Fragment,
    useCallback
} from "react";
import { getCodeString } from "rehype-rewrite";
import mermaid from "mermaid";

const randomid = () => parseInt(String(Math.random() * 1e15), 10).toString(36);
export const Code = ({ inline, children = [], className, ...props }) => {
    const demoid = useRef(`dome${randomid()}`);
    const [container, setContainer] = useState(null);
    const isMermaid =
        className && /^language-mermaid/.test(className.toLocaleLowerCase());
    const code =
        props.node && props.node.children
            ? getCodeString(props.node.children)
            : children[0] || "";

    const reRender = async () => {
        if (container && isMermaid) {
            try {
                const str = await mermaid.render(demoid.current, code);
                container.innerHTML = str.svg;
            } catch (error) {
                container.innerHTML = error;
            }
        }
    };

    useEffect(() => {
        reRender();
    }, [container, isMermaid, code, demoid]);

    const refElement = useCallback((node) => {
        if (node !== null) {
            setContainer(node);
        }
    }, []);

    if (isMermaid) {
        return (
            <Fragment>
                <code id={demoid.current} style={{ display: "none" }} />
                <code ref={refElement} data-name="mermaid" />
            </Fragment>
        );
    }
    return <code>{children}</code>;
};
