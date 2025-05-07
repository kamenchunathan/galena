// dom-renderer.ts

type AttributePair = [string, string];

interface TextNode {
  Text: string;
}

interface InputNode {
  Input: {
    attributes: AttributePair[];
  };
}

interface DivNode {
  Div: {
    attributes: AttributePair[];
    children: (DivNode | TextNode | InputNode)[];
  };
}

type ViewNode = DivNode | TextNode | InputNode;

/**
 * Main renderer function that takes the JSON output from the view function
 * and creates a DOM tree
 */
export function renderViewToDOM(
  rootElement: HTMLElement,
  viewJson: ViewNode,
): void {
  // Clear any existing content
  rootElement.innerHTML = "";

  // Append the new DOM tree
  const domNode = createDOMNode(viewJson);
  if (domNode) {
    rootElement.appendChild(domNode);
  }
}

/**
 * Creates a DOM node from a ViewNode
 */
function createDOMNode(node: ViewNode): Node | null {
  // Handle Text nodes
  if ("Text" in node) {
    return document.createTextNode(node.Text);
  }

  // Handle Input nodes
  if ("Input" in node) {
    const inputElement = document.createElement("input");
    applyAttributes(inputElement, node.Input.attributes);
    return inputElement;
  }

  // Handle Div nodes
  if ("Div" in node) {
    const divElement = document.createElement("div");
    applyAttributes(divElement, node.Div.attributes);

    // Process children recursively
    if (node.Div.children && Array.isArray(node.Div.children)) {
      node.Div.children.forEach((childNode) => {
        const childDomNode = createDOMNode(childNode);
        if (childDomNode) {
          divElement.appendChild(childDomNode);
        }
      });
    }

    return divElement;
  }

  console.error("Unknown node type:", node);
  return null;
}

/**
 * Applies attributes to a DOM element
 */
function applyAttributes(
  element: HTMLElement,
  attributes: AttributePair[],
): void {
  attributes.forEach(([key, value]) => {
    if (key === "class") {
      element.className = value;
    } else if (key === "val" && element instanceof HTMLInputElement) {
      element.value = value;
    } else {
      element.setAttribute(key, value);
    }
  });
}
