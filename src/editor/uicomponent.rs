use std::io::Error;

use super::terminal::Size;

pub trait UIComponent {
    /// Updates the sizes and marks as redraw-needed
    fn resize(&mut self, to: Size) {
        self.set_size(to);
        self.mark_redraw(true);
    }

    /// Draw this component if it's visible and in need of re-drawing
    fn render(&mut self, origin_y: usize) {
        if self.needs_redraw() {
            match self.draw(origin_y) {
                Ok(()) => self.mark_redraw(false),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not render component: {err:?}");
                    }
                }
            }
        }
    }

    /// Method to actually draw the component, must be implemented by each component
    fn draw(&mut self, origin_y: usize) -> Result<(), Error>;

    /// Updates the size. Needs to be implemented by each component.
    fn set_size(&mut self, to: Size);

    /// Marks this UI component as in need of redrawing (or not)
    fn mark_redraw(&mut self, value: bool);

    /// Determines if a component needs to be redrawn or not
    fn needs_redraw(&mut self) -> bool;
}
